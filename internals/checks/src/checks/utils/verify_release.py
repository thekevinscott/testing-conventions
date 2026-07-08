"""Validated-promotion orchestration: verify a release before `@v0` advances — repo-only (#357).

The logic behind `tc-checks verify-release` (cli.py is a thin wrapper). Repo-only: not shipped in
the published tool; invoked by the verify-and-promote workflow (`move-major-tag.yml`) after a
successful `Release`, gating the forward-only tag move on a green verification of the exact
combination the tag is about to bless — the *new* workflow file, the *published* binary, the
*current* `@v0` detect. See `internals/repo.md`, "Validated promotion".

Three operations the workflow wires as separate steps:

  resolve_version(sha)              the just-published npm version to pin the verification to — the
                                    highest `testing-conventions-npm-v*` tag reachable from the
                                    release commit (putitoutthere tags on publish).
  check_layout(sha)                 the paths a consumer's remote `detect` action fetch resolves
                                    (`action.yml` and the `detect.py` it reaches) must be present
                                    in `git archive <sha>` — the fetch/layout provenance the
                                    version-pinned suite can't cover (it runs `detect@v0`).
  verify_suites(sha, version, wfs)  dispatch each workflow at the release commit (via a throwaway
                                    tag, since `workflow_dispatch` takes a branch/tag ref, not a
                                    SHA) with the pinned version and poll conclusions.

The one external — git + the `gh` CLI — is the injected `run` (defaulting to `subprocess.run`),
the `stage_hermetic_cli`/`run_checks` pattern (#328, #356): the colocated test drives a fake that
dispatches on argv, so every parse and decision runs with no real git/gh, while the workflow runs
them for real. Time is injected too (`sleep`/`clock`/`now`) so the poll loop is deterministic.
"""
from __future__ import annotations

import json
import subprocess
import time
from datetime import datetime, timezone

from checks.utils.check_failed import CheckFailed

# The remote-fetch targets a consumer's `detect` composite action resolves: GitHub fetches the
# repo at `@v0` and runs `action.yml`, which reaches its implementation via
# `$GITHUB_ACTION_PATH/../../../internals/detect/src/detect.py` (#363). Both must survive into the
# archived tree of the promoted commit, or every consumer's detect job dies the moment `@v0` moves.
REQUIRED_ACTION_PATHS = (
    ".github/actions/detect/action.yml",
    "internals/detect/src/detect.py",
)

NPM_TAG_PREFIX = "testing-conventions-npm-v"

# The throwaway tag naming the release commit for dispatch (created at the release SHA, deleted
# after). The SHA in the name keeps concurrent verifications from colliding on the ref.
TEMP_TAG_PREFIX = "verify-release-"

# How long to wait for a dispatched run to register, and how often to poll.
RUN_APPEAR_TIMEOUT_S = 120
RUN_POLL_INTERVAL_S = 10


# --- pure decisions ---

def published_version(tags: list[str]) -> str:
    """The highest npm version among `testing-conventions-npm-v*` tags, as a bare `X.Y.Z`.

    Sorted by numeric (major, minor, patch), so `v0.0.9` sorts below `v0.0.67` (a lexical sort
    would invert them). Raises when no npm tag is present — the release published no binary to pin
    to, so there is nothing to verify against and the promotion must not proceed (fail closed).
    """
    versions = [tag[len(NPM_TAG_PREFIX):] for tag in tags if tag.startswith(NPM_TAG_PREFIX)]
    if not versions:
        raise ValueError(
            f"no {NPM_TAG_PREFIX}* tag reachable from the release commit — no published npm "
            "version to pin the verification to; refusing to promote (#357)"
        )
    return max(versions, key=_version_key)


def _version_key(version: str) -> tuple[int, ...]:
    return tuple(int(part) for part in version.split("."))


def missing_paths(present: set[str], required: tuple[str, ...] = REQUIRED_ACTION_PATHS) -> list[str]:
    """The `required` paths absent from `present`, in the given order."""
    return [path for path in required if path not in present]


def select_dispatched_run(runs: list[dict], sha: str, since: str) -> dict:
    """The newest `workflow_dispatch` run at `sha` created at/after `since` (an ISO-8601 UTC ts).

    Filtering by head SHA *and* dispatch event *and* the pre-dispatch timestamp pins the choice to
    the run this verification just triggered, not a stale run of the same workflow at the same
    commit. Raises `LookupError` when none matches yet (the run hasn't registered — caller retries).
    """
    matching = [
        run
        for run in runs
        if run.get("headSha") == sha
        and run.get("event") == "workflow_dispatch"
        and run.get("createdAt", "") >= since
    ]
    if not matching:
        raise LookupError("no dispatched run registered yet for this sha")
    return max(matching, key=lambda run: run["createdAt"])


def failed_suites(conclusions: dict) -> list[str]:
    """The `workflow (conclusion)` labels whose conclusion is not `success` — the fail-closed set."""
    return [
        f"{workflow} ({conclusion or 'no conclusion'})"
        for workflow, conclusion in conclusions.items()
        if conclusion != "success"
    ]


def now_iso() -> str:
    """The current time as an ISO-8601 UTC timestamp, matching GitHub's `createdAt` format."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# --- operations (git + gh through the injected `run`) ---

def _ensure_ok(result, argv: list[str]) -> None:
    """Raise unless `result` (a completed subprocess) exited zero — a nonzero (incl. signal) code raises."""
    if result.returncode:
        raise CheckFailed(f"`{' '.join(argv)}` exited {result.returncode}")


def _run_text(run, argv: list[str]) -> str:
    result = run(argv, capture_output=True, text=True)  # boundary flags: exempt (mutation) — see testing-conventions.toml
    _ensure_ok(result, argv)
    return result.stdout


def _run_bytes(run, argv: list[str], **extra) -> bytes:
    result = run(argv, capture_output=True, **extra)  # boundary flag: exempt (mutation) — see testing-conventions.toml
    _ensure_ok(result, argv)
    return result.stdout


def resolve_version(sha: str, run=subprocess.run) -> str:
    """The just-published npm version pinned from the `testing-conventions-npm-v*` tags at `sha`."""
    out = _run_text(run, ["git", "tag", "--merged", sha, "--list", f"{NPM_TAG_PREFIX}*"])
    return published_version([line.strip() for line in out.splitlines() if line.strip()])


def check_layout(sha: str, run=subprocess.run) -> list[str]:
    """The required `detect`-action paths absent from `git archive <sha>` — empty when all present."""
    archive = _run_bytes(run, ["git", "archive", "--format=tar", sha])
    names = _run_bytes(run, ["tar", "--list", "--file", "-"], input=archive).decode()
    return missing_paths({line.rstrip("/") for line in names.splitlines() if line})


def verify_suites(
    sha: str,
    version: str,
    workflows: list[str],
    run=subprocess.run,
    *,
    sleep=time.sleep,
    clock=time.monotonic,
    now=now_iso,
) -> dict:
    """Dispatch every workflow in `workflows` at `sha` with `version`; return `{workflow: conclusion}`.

    Creates one throwaway tag at `sha` (the dispatch ref — `workflow_dispatch` takes a branch/tag,
    never a bare SHA), dispatches all workflows at it *before* awaiting any so they run in parallel,
    then watches each to completion. The tag is deleted in a `finally`, so a failure or timeout
    still cleans it up. A pre-dispatch timestamp pins each lookup to the run just triggered.
    """
    tag = f"{TEMP_TAG_PREFIX}{sha}"
    _run_bytes(run, ["git", "push", "origin", f"{sha}:refs/tags/{tag}"])
    try:
        since = now()
        for workflow in workflows:
            _run_bytes(run, ["gh", "workflow", "run", workflow, "--ref", tag, "-f", f"version={version}"])
        run_ids = {
            workflow: _await_run(workflow, sha, since, run, sleep, clock)
            for workflow in workflows
        }
        return {workflow: _watch_conclusion(run_id, run, sleep) for workflow, run_id in run_ids.items()}
    finally:
        _run_bytes(run, ["git", "push", "origin", f":refs/tags/{tag}"])


def _list_runs(workflow: str, run) -> list[dict]:
    out = _run_text(run, [
        "gh", "run", "list", "--workflow", workflow, "--limit", "40",
        "--json", "databaseId,headSha,event,status,conclusion,createdAt",
    ])
    return json.loads(out)


def _await_run(workflow: str, sha: str, since: str, run, sleep, clock) -> int:
    """Poll `workflow`'s runs until the dispatched one at `sha` (created at/after `since`) registers."""
    deadline = clock() + RUN_APPEAR_TIMEOUT_S
    while True:
        try:
            return select_dispatched_run(_list_runs(workflow, run), sha, since)["databaseId"]
        except LookupError:
            if clock() >= deadline:
                raise TimeoutError(
                    f"dispatched run for {workflow} at {sha} never registered within "
                    f"{RUN_APPEAR_TIMEOUT_S}s (#357)"
                )
            sleep(RUN_POLL_INTERVAL_S)


def _watch_conclusion(run_id: int, run, sleep) -> str:
    """Block until run `run_id` completes; return its conclusion (e.g. 'success', 'failure')."""
    while True:
        state = json.loads(_run_text(run, ["gh", "run", "view", str(run_id), "--json", "status,conclusion"]))
        if state.get("status") == "completed":
            return state.get("conclusion") or ""
        sleep(RUN_POLL_INTERVAL_S)
