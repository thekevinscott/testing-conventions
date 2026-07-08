#!/usr/bin/env python3
"""Validated-promotion helper: verify a release before `@v0` advances — repo-only (#357).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it is invoked by the `Move major tag` (verify-and-promote) workflow
after a successful `Release`, and gates the forward-only tag move (`move_major_tag.py`) on a
green verification of the exact combination the tag is about to bless — the *new* workflow file,
the *published* binary, the *current* `@v0` detect. See `internals/repo.md`,
"Validated promotion: verify before `@v0` advances".

Three operations, each a thin `main()` subcommand so the workflow can wire them as separate
steps and capture their output:

  resolve-version <sha>              print the just-published npm version to pin the verification
                                     to — the highest `testing-conventions-npm-v*` tag reachable
                                     from the release commit (putitoutthere tags on publish).
  check-layout <sha>                 assert the paths a consumer's remote `detect` action fetch
                                     resolves (`action.yml` and the `detect.py` it reaches) are
                                     present in `git archive <sha>` — the fetch/layout provenance
                                     the version-pinned suite can't cover (it runs `detect@v0`,
                                     the old tag). Fails closed if any is missing.
  dispatch-and-wait <wf> <sha> <v>   dispatch the workflow at `ref: <sha>` with `version=<v>` and
                                     poll its conclusion; exit non-zero unless it succeeds.

The two externals — git (archive, tag listing) and the `gh` CLI (dispatch, run listing/watching)
— live behind the small boundary functions below, so the integration suite mocks them and
exercises the real orchestration, while the e2e suite runs the git-backed operations against a
real repo. The pure decisions (`published_version`, `missing_paths`, `select_dispatched_run`) are
unit-tested in isolation.
"""
from __future__ import annotations

import subprocess
import sys
import time
from datetime import datetime, timezone

# The remote-fetch targets a consumer's `detect` composite action resolves: GitHub fetches the
# repo at `@v0` and runs `action.yml`, which reaches its implementation via
# `$GITHUB_ACTION_PATH/../../../internals/detect/src/detect.py` (#363). Both must survive into the
# archived tree of the promoted commit, or every consumer's detect job dies the moment `@v0` moves.
REQUIRED_ACTION_PATHS = (
    ".github/actions/detect/action.yml",
    "internals/detect/src/detect.py",
)

NPM_TAG_PREFIX = "testing-conventions-npm-v"

# How long to wait for a dispatched run to register and to conclude.
_RUN_APPEAR_TIMEOUT_S = 120
_RUN_POLL_INTERVAL_S = 10


# --- pure decisions (no git, no gh — unit-tested in isolation) ---

def published_version(tags: list[str]) -> str:
    """The highest npm version among `testing-conventions-npm-v*` tags, as a bare `X.Y.Z`.

    Sorted by numeric (major, minor, patch), so `v0.0.9` sorts below `v0.0.67` (a lexical sort
    would invert them). Raises when no npm tag is present — the release published no binary to
    pin the verification to, so there is nothing to verify against and the promotion must not
    proceed (fail closed).
    """
    versions = [t[len(NPM_TAG_PREFIX):] for t in tags if t.startswith(NPM_TAG_PREFIX)]
    if not versions:
        raise ValueError(
            f"no {NPM_TAG_PREFIX}* tag reachable from the release commit — no published npm "
            "version to pin the verification to; refusing to promote (#357)"
        )
    return max(versions, key=_version_key)


def _version_key(version: str) -> tuple[int, ...]:
    return tuple(int(part) for part in version.split("."))


def missing_paths(archive_paths: set[str], required: tuple[str, ...] = REQUIRED_ACTION_PATHS) -> list[str]:
    """The `required` paths absent from `archive_paths`, in the given order."""
    return [path for path in required if path not in archive_paths]


def select_dispatched_run(runs: list[dict], sha: str, since: str) -> dict:
    """The newest `workflow_dispatch` run at `sha` created at/after `since` (an ISO-8601 UTC ts).

    Filtering by head SHA *and* dispatch event *and* the pre-dispatch timestamp pins the choice to
    the run this verification just triggered, not a stale run of the same workflow at the same
    commit. Raises when none matches yet (the run hasn't registered — the caller retries).
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


# --- git / gh boundary (the externals the integration suite mocks) ---

def archive_paths(sha: str) -> set[str]:
    """Every path in `git archive <sha>` — exactly the tree a remote action fetch would see."""
    listing = subprocess.run(
        ["git", "archive", "--format=tar", sha],
        capture_output=True, check=True,
    ).stdout
    names = subprocess.run(
        ["tar", "--list", "--file", "-"],
        input=listing, capture_output=True, check=True,
    ).stdout.decode()
    return {line.rstrip("/") for line in names.splitlines() if line}


def reachable_npm_tags(sha: str) -> list[str]:
    """The `testing-conventions-npm-v*` tags reachable from (merged into) `sha`."""
    out = subprocess.run(
        ["git", "tag", "--merged", sha, "--list", f"{NPM_TAG_PREFIX}*"],
        capture_output=True, text=True, check=True,
    ).stdout
    return [line.strip() for line in out.splitlines() if line.strip()]


def dispatch(workflow: str, ref: str, version: str, run=subprocess.run) -> None:
    """`gh workflow run <workflow> --ref <ref> -f version=<version>`."""
    run(
        ["gh", "workflow", "run", workflow, "--ref", ref, "-f", f"version={version}"],
        check=True,
    )


def list_runs(workflow: str, run=subprocess.run) -> list[dict]:
    """The recent runs of `workflow` as dicts (databaseId, headSha, event, status, conclusion, createdAt)."""
    import json

    out = run(
        ["gh", "run", "list", "--workflow", workflow, "--limit", "40",
         "--json", "databaseId,headSha,event,status,conclusion,createdAt"],
        capture_output=True, text=True, check=True,
    ).stdout
    return json.loads(out)


def watch_conclusion(run_id: int, run=subprocess.run) -> str:
    """Block until run `run_id` completes; return its conclusion (e.g. 'success', 'failure')."""
    while True:
        import json

        out = run(
            ["gh", "run", "view", str(run_id), "--json", "status,conclusion"],
            capture_output=True, text=True, check=True,
        ).stdout
        state = json.loads(out)
        if state.get("status") == "completed":
            return state.get("conclusion") or ""
        time.sleep(_RUN_POLL_INTERVAL_S)


def now_iso() -> str:
    """The current time as an ISO-8601 UTC timestamp, matching GitHub's `createdAt` format."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


# --- orchestration (real; only the boundary is mocked in integration) ---

def dispatch_and_wait(workflow: str, sha: str, version: str, *, sleep=time.sleep) -> str:
    """Dispatch `workflow` at `sha` with `version`, find the run it created, and return its conclusion.

    Records a pre-dispatch timestamp so `select_dispatched_run` can pin to *this* run, then polls
    the run list until it registers (a dispatch is asynchronous) before watching it to completion.
    """
    since = now_iso()
    dispatch(workflow, sha, version)
    deadline = time.monotonic() + _RUN_APPEAR_TIMEOUT_S
    while True:
        try:
            run = select_dispatched_run(list_runs(workflow), sha, since)
            break
        except LookupError:
            if time.monotonic() >= deadline:
                raise TimeoutError(
                    f"dispatched run for {workflow} at {sha} never registered within "
                    f"{_RUN_APPEAR_TIMEOUT_S}s (#357)"
                )
            sleep(_RUN_POLL_INTERVAL_S)
    return watch_conclusion(run["databaseId"])


def main(argv: list[str]) -> int:
    command, args = argv[1], argv[2:]
    if command == "resolve-version":
        (sha,) = args
        print(published_version(reachable_npm_tags(sha)))
        return 0
    if command == "check-layout":
        (sha,) = args
        absent = missing_paths(archive_paths(sha))
        if absent:
            print(
                f"::error::the release archive of {sha} is missing "
                + ", ".join(absent)
                + " — a consumer's remote `detect` action fetch would resolve a broken action the "
                "moment @v0 moves; refusing to promote (#357)"
            )
            return 1
        print(f"detect action layout present in the archive of {sha}")
        return 0
    if command == "dispatch-and-wait":
        workflow, sha, version = args
        conclusion = dispatch_and_wait(workflow, sha, version)
        if conclusion != "success":
            print(f"::error::{workflow} at {sha} concluded '{conclusion}', not success; refusing to promote (#357)")
            return 1
        print(f"{workflow} at {sha} passed the version-pinned verification")
        return 0
    print(f"::error::unknown command {command!r}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
