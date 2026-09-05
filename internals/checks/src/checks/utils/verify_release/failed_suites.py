"""The fail-closed reading of a set of suite conclusions."""
from __future__ import annotations


def failed_suites(conclusions: dict) -> list[str]:
    """The `workflow (conclusion)` labels whose conclusion is not `success`."""
    return [
        f"{workflow} ({conclusion or 'no conclusion'})"
        for workflow, conclusion in conclusions.items()
        if conclusion != "success"
    ]
