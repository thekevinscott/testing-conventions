"""End-to-end test for check_mutation_wired: runs the real `__main__` entry via runpy.

Covers `main`, the file read, and the `__main__` guard (the lines the pure unit test does not
touch) against real fixture files on disk — a wired one and a broken one — asserting the exit
code and the printed line each way.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_mutation_wired.py"


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


def test_e2e_passes_on_a_wired_workflow(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text('run: unit mutation --language "$LANGUAGE" --base "$BASE"\n')
    assert run(["prog", str(wf)]) == 0
    assert "invokes `unit mutation`" in capsys.readouterr().out


def test_e2e_fails_on_an_unwired_workflow(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text('run: unit coverage --language "$LANGUAGE"\n')
    assert run(["prog", str(wf)]) == 1
    assert "::error::" in capsys.readouterr().out
