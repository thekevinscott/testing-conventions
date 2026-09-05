"""Colocated unit tests for the decoded-run helper (isolation — an injected `run` fake)."""
from checks.utils.verify_release.run_text import run_text


class _Result:
    def __init__(self, stdout="", returncode=0):
        self.stdout = stdout
        self.returncode = returncode


def test_run_text_asks_subprocess_to_capture_and_decode():
    seen = []

    def run(argv, **kwargs):
        seen.append((argv, kwargs))
        return _Result(stdout="the output\n")

    assert run_text(run, ["git", "status"]) == "the output\n"
    assert seen == [(["git", "status"], {"capture_output": True, "text": True})]


def test_run_text_raises_when_the_command_exits_nonzero():
    def run(argv, **kwargs):
        return _Result(returncode=2)

    try:
        run_text(run, ["git", "status"])
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "exited 2" in error.message
    else:
        raise AssertionError("a failing command must raise")
