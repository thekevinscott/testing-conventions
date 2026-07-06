"""End-to-end tests for the isolation-red check: the real entry point, no mocks.

Per the standard, an e2e test runs with no mocks. It drives the script's `__main__` entry in-process
via `runpy` — with the command supplied as `sys.argv`, using the benign shell builtins `true`
(exit 0) and `false` (exit 1) in place of the real npx invocation — so the whole run -> decide ->
report path, the real `run_command` subprocess boundary, and the `__main__` guard are all measured.
`sys.argv` is set and restored around each run, confined to the helper.

The run name is a distinct `"__main__"` object (built at runtime, not the interned literal), so the
guard `__name__ == "__main__"` is exercised by *value* equality: it still fires (the object equals
`"__main__"`), and an identity (`is`) or ordering (`<=`) mutation of that comparison would change
whether `main` runs — caught by the assertions below and the not-main run.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_isolation_red.py"

# Equal to "__main__" by value but a distinct object, so the guard's equality is tested (not identity).
MAIN_BY_VALUE = "".join(["__mai", "n__"])


def run(argv, run_name=MAIN_BY_VALUE):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name=run_name)
        return 0
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old


def test_e2e_red_check_passes_when_the_command_fails(capsys):
    # A command that exits non-zero -> the red-path assertion holds -> exit 0.
    assert run(["check_isolation_red.py", "false"]) == 0
    assert "[cli] ok" in capsys.readouterr().out


def test_e2e_red_check_fails_when_the_command_passes(capsys):
    # A command that exits 0 -> the red-path assertion is violated -> exit 1.
    assert run(["check_isolation_red.py", "true"]) == 1
    assert "::error::[cli]" in capsys.readouterr().out


def test_e2e_guard_stays_silent_when_not_run_as_main(capsys):
    # A run name below "__main__" makes the guard's `==` false, so main never runs. An ordering
    # mutation (`__name__ <= "__main__"`) would sort this name in and run main — caught here.
    assert run(["check_isolation_red.py", "false"], run_name="__aaa__") == 0
    assert "[cli]" not in capsys.readouterr().out
