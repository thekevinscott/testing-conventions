"""Colocated unit tests for the raw-bytes run helper (isolation — an injected `run` fake)."""
from checks.utils.verify_release.run_bytes import run_bytes


class _Result:
    def __init__(self, stdout=b"", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_run_bytes_captures_without_decoding_and_forwards_its_extras():
    seen = []

    def run(argv, **kwargs):
        seen.append((argv, kwargs))
        return _Result(stdout=b"tar-bytes")

    assert run_bytes(run, ["git", "archive"], input=b"stdin-bytes") == b"tar-bytes"
    assert seen == [(["git", "archive"], {"capture_output": True, "input": b"stdin-bytes"})]


def test_run_bytes_raises_when_the_command_exits_nonzero():
    def run(argv, **kwargs):
        return _Result(returncode=3)

    try:
        run_bytes(run, ["git", "archive"])
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "exited 3" in error.message
    else:
        raise AssertionError("a failing command must raise")
