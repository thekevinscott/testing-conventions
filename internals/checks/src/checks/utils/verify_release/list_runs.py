"""List a workflow's recent runs through `gh`."""
from __future__ import annotations

import json

from checks.utils.verify_release.run_text import run_text


def list_runs(workflow: str, run) -> list[dict]:
    out = run_text(run, [
        "gh", "run", "list", "--workflow", workflow, "--limit", "40",
        "--json", "databaseId,headSha,event,status,conclusion,createdAt",
    ])
    return json.loads(out)
