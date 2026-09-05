"""Watch a workflow run to completion."""
from __future__ import annotations

import json

from checks.utils.verify_release.await_run import RUN_POLL_INTERVAL_S
from checks.utils.verify_release.run_text import run_text


def watch_conclusion(run_id: int, run, sleep) -> str:
    """Block until run `run_id` completes; return its conclusion (e.g. 'success', 'failure')."""
    while True:
        state = json.loads(run_text(run, ["gh", "run", "view", str(run_id), "--json", "status,conclusion"]))
        if state.get("status") == "completed":
            return state.get("conclusion") or ""
        sleep(RUN_POLL_INTERVAL_S)
