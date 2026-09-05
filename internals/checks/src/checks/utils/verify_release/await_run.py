"""Wait for a dispatched workflow run to register."""
from __future__ import annotations

from checks.utils.verify_release.list_runs import list_runs
from checks.utils.verify_release.select_dispatched_run import select_dispatched_run

RUN_APPEAR_TIMEOUT_S = 120
RUN_POLL_INTERVAL_S = 10


def await_run(workflow: str, sha: str, since: str, run, sleep, clock) -> int:
    """Poll `workflow`'s runs until the dispatched one at `sha` (created at/after `since`) registers."""
    deadline = clock() + RUN_APPEAR_TIMEOUT_S
    while True:
        try:
            return select_dispatched_run(list_runs(workflow, run), sha, since)["databaseId"]
        except LookupError:
            if clock() >= deadline:
                raise TimeoutError(
                    f"dispatched run for {workflow} at {sha} never registered within "
                    f"{RUN_APPEAR_TIMEOUT_S}s (#357)"
                )
            sleep(RUN_POLL_INTERVAL_S)
