"""Dispatch the verification suites at a release commit and collect their conclusions."""
from __future__ import annotations

import subprocess
import time
from datetime import datetime, timezone

from checks.utils.verify_release.await_run import await_run
from checks.utils.verify_release.run_bytes import run_bytes
from checks.utils.verify_release.watch_conclusion import watch_conclusion

# The SHA in the tag name keeps concurrent verifications from colliding on the ref.
TEMP_TAG_PREFIX = "verify-release-"


def now_iso() -> str:
    """The current time as an ISO-8601 UTC timestamp, matching GitHub's `createdAt` format."""
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


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

    One throwaway tag at `sha` is the dispatch ref (`workflow_dispatch` takes a branch/tag, never a
    bare SHA); all workflows dispatch before any await, and a `finally` deletes the tag either way."""
    tag = f"{TEMP_TAG_PREFIX}{sha}"
    run_bytes(run, ["git", "push", "origin", f"{sha}:refs/tags/{tag}"])
    try:
        since = now()
        for workflow in workflows:
            run_bytes(run, ["gh", "workflow", "run", workflow, "--ref", tag, "-f", f"version={version}"])
        run_ids = {
            workflow: await_run(workflow, sha, since, run, sleep, clock)
            for workflow in workflows
        }
        return {workflow: watch_conclusion(run_id, run, sleep) for workflow, run_id in run_ids.items()}
    finally:
        run_bytes(run, ["git", "push", "origin", f":refs/tags/{tag}"])
