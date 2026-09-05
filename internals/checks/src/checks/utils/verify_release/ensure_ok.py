"""The exit-code guard every git/gh invocation runs through."""
from __future__ import annotations

from checks.utils.check_failed import CheckFailed


def ensure_ok(result, argv: list[str]) -> None:
    """Raise unless `result` (a completed subprocess) exited zero — a nonzero (incl. signal) code raises."""
    if result.returncode:
        raise CheckFailed(f"`{' '.join(argv)}` exited {result.returncode}")
