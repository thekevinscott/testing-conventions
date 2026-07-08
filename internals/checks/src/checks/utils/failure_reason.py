"""The pure exit-code decision shared by the failure-path checks (#328).

`failure_reason(expect_fail, returncode)` returns `None` when `returncode` met the expectation, else
the reason it did not. Truthiness — not `== 0` — so no comparison operator can mutate into an
equivalent (`<= 0` never differs for a real exit code; `is 0` never differs for a small cached int).
"""
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
