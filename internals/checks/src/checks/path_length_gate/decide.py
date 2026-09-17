"""The path-length-gate budget decision — repo-only.

See docs/internals/repo.md, "Windows checkout path budget", for the 200 derivation.
"""
from __future__ import annotations

BUDGET = 200


def over_budget(paths, budget: int = BUDGET) -> list[tuple[str, int]]:
    """Every (path, length) over `budget`, longest first, then alphabetical."""
    violations = [(path, len(path)) for path in paths if len(path) > budget]
    violations.sort(key=lambda pair: pair[0])
    violations.sort(key=lambda pair: pair[1], reverse=True)
    return violations
