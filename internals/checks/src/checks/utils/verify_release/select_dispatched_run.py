"""Pick the run a dispatch just triggered out of a workflow's run list."""
from __future__ import annotations


def select_dispatched_run(runs: list[dict], sha: str, since: str) -> dict:
    """The newest `workflow_dispatch` run at `sha` created at/after `since` (an ISO-8601 UTC ts) —
    sha, event, and pre-dispatch timestamp together pin the run this verification just triggered.
    Raises `LookupError` when none matches yet (the run hasn't registered — caller retries)."""
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
