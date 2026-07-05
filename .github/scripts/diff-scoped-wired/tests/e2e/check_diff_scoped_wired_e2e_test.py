"""End-to-end test for check_diff_scoped_wired: runs the real `__main__` entry via runpy.

Covers `main`, the file read, and the `__main__` guard against real fixture files — one with both
`--base` checks and one missing them — asserting exit code and printed line each way.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_diff_scoped_wired.py"


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


def test_e2e_passes_when_both_base_checks_present(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(
        'run: unit colocated-test --language python --base "$BASE"\n'
        'run: unit coverage --language python --base "$BASE"\n'
    )
    assert run(["prog", str(wf)]) == 0
    assert "diff-scoped --base checks" in capsys.readouterr().out


def test_e2e_fails_when_a_base_check_is_missing(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text('run: unit coverage --language python\n')
    assert run(["prog", str(wf)]) == 1
    assert "::error::" in capsys.readouterr().out
