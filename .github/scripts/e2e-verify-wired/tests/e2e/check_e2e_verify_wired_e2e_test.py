"""End-to-end test for check_e2e_verify_wired: runs the real `__main__` entry via runpy.

Covers `main`, the file read, and the `__main__` guard against real fixture files — one wiring
both the input and command, one missing them — asserting exit code and printed line each way.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_e2e_verify_wired.py"


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


def test_e2e_passes_when_input_and_command_present(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text("  run_e2e:\n    description: force it\nrun: e2e verify \"$ROOT\"\n")
    assert run(["prog", str(wf)]) == 0
    assert "opt-in e2e verify" in capsys.readouterr().out


def test_e2e_fails_when_wiring_missing(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text("run: unit coverage --language python\n")
    assert run(["prog", str(wf)]) == 1
    assert "::error::" in capsys.readouterr().out
