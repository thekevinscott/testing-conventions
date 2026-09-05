"""Colocated unit tests for the exit-code guard (isolation — a stub result, no subprocess)."""
from checks.utils.verify_release.ensure_ok import ensure_ok


class _Result:
    def __init__(self, returncode=0):
        self.returncode = returncode


def test_a_zero_exit_passes_silently():
    assert ensure_ok(_Result(returncode=0), ["git", "status"]) is None


def test_a_nonzero_exit_raises_naming_the_command_and_code():
    try:
        ensure_ok(_Result(returncode=2), ["git", "tag", "--merged"])
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert error.message == "`git tag --merged` exited 2"
    else:
        raise AssertionError("a failing command must raise")


def test_a_signal_death_raises_too():
    # A negative return code (POSIX signal, e.g. OOM-killed git) is nonzero and must raise too.
    try:
        ensure_ok(_Result(returncode=-9), ["git", "archive"])
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "exited -9" in error.message
    else:
        raise AssertionError("a signal-killed command must raise")
