"""Colocated unit test for `run_checks` — the shared failure-path orchestrator (isolation).

The subprocess boundary is injected as `run`, so a fake stands in for it — no real process. The
raise path is asserted against the propagated exception's `.message`, so `CheckFailed` is never
imported as a collaborator.
"""
from checks.utils.run_checks import run_checks


class _Result:
    """The slice of `subprocess.CompletedProcess` the orchestrator reads."""

    def __init__(self, returncode):
        self.returncode = returncode


def _run_returning(returncode):
    """A fake `run` that records the argv it saw and returns a fixed exit code."""
    seen = []

    def run(argv):
        seen.append(argv)
        return _Result(returncode)

    run.seen = seen
    return run


def test_echoes_ok_for_every_check_that_holds(capsys):
    # A red check whose command fails, plus a clean check whose command passes — both hold.
    fake = _run_returning(0)
    run_checks(
        [(["a"], False, "clean"), (["b"], False, "also-clean")],
        run=fake,
    )
    out = capsys.readouterr().out
    assert "[clean] ok" in out
    assert "[also-clean] ok" in out
    assert fake.seen == [["a"], ["b"]]


def test_raises_when_a_red_check_passed(capsys):
    # expect_fail=True but the command exited 0 -> the red-path expectation is violated.
    try:
        run_checks([(["x"], True, "must-fail")], run=_run_returning(0))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "[must-fail]" in error.message
        assert "non-zero" in error.message
    else:
        raise AssertionError("a red check that passed must raise")


def test_raises_when_a_clean_check_failed():
    # expect_fail=False but the command exited non-zero -> the clean expectation is violated.
    try:
        run_checks([(["x"], False, "must-pass")], run=_run_returning(2))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "[must-pass]" in error.message
        assert "exited 2" in error.message
    else:
        raise AssertionError("a clean check that failed must raise")


def test_stops_at_the_first_violation():
    # The second check never runs once the first raises.
    fake = _run_returning(0)
    try:
        run_checks([(["first"], True, "first"), (["second"], True, "second")], run=fake)
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party
        assert "[first]" in error.message
        assert fake.seen == [["first"]]
    else:
        raise AssertionError("the first violation must raise before the second check runs")


def test_a_trailing_command_becomes_a_single_red_path_check(capsys):
    # A trailing command replaces CHECKS with one red-path check labelled "cli"; a non-zero exit
    # makes that expectation hold.
    fake = _run_returning(1)
    run_checks([(["ignored"], False, "ignored")], command=("npx", "run"), run=fake)
    out = capsys.readouterr().out
    assert "[cli] ok" in out
    assert "ignored" not in out
    assert fake.seen == [["npx", "run"]]
