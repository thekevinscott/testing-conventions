"""End-to-end tests for the detect-routes-python helper: the real `__main__` entry, no mocks.

Runs the script's `__main__` entry point in-process via `runpy`, passing an `isolation_languages`
value as `argv[1]` exactly as the workflow does. Covers `main`, the argv read, and the `__main__`
guard. A value that routes Python in exits 0; one that does not exits 1.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_detect_routes_python.py"


def run(argv):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name="__main__")
        return 0
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old


def test_e2e_passes_when_python_is_routed_in(capsys):
    assert run(["check.py", '["python","rust"]']) == 0
    out = capsys.readouterr().out
    assert 'isolation_languages=["python","rust"]' in out
    assert "Python routed into the unit-lint matrix" in out


def test_e2e_fails_when_python_is_absent(capsys):
    assert run(["check.py", '["rust"]']) == 1
    assert "::error::" in capsys.readouterr().out


def test_e2e_fails_with_no_argument(capsys):
    # No argv[1]: the empty default does not route Python in.
    assert run(["check.py"]) == 1
    assert "::error::" in capsys.readouterr().out
