"""Colocated unit test for `failure_reason` — the pure exit-code decision (isolation).

Driven directly over the four (expect_fail, returncode) quadrants; no subprocess, no click.
"""
from checks.utils.failure_reason import failure_reason


def test_red_check_flags_a_zero_exit():
    # expect_fail=True, a zero exit is the violation.
    assert failure_reason(True, 0) == "the command exited 0, but a non-zero (failing) exit was required"


def test_red_check_passes_a_nonzero_exit():
    # Any truthy (non-zero) exit clears a red-path check — including a signal-kill (negative).
    assert failure_reason(True, 1) is None
    assert failure_reason(True, -9) is None


def test_clean_check_passes_a_zero_exit():
    assert failure_reason(False, 0) is None


def test_clean_check_flags_a_nonzero_exit():
    assert failure_reason(False, 3) == "the command exited 3, but a zero (passing) exit was required"
    assert failure_reason(False, -9) == "the command exited -9, but a zero (passing) exit was required"
