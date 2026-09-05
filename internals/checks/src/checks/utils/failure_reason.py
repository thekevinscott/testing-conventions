"""The pure exit-code decision shared by the failure-path checks — truthiness, not `== 0`, so no
comparison mutant of it is behaviorally identical."""
from __future__ import annotations

from typing import Optional


def failure_reason(expect_fail: bool, returncode: int) -> Optional[str]:
    if expect_fail:
        if not returncode:
            return "the command exited 0, but a non-zero (failing) exit was required"
        return None
    if returncode:
        return f"the command exited {returncode}, but a zero (passing) exit was required"
    return None
