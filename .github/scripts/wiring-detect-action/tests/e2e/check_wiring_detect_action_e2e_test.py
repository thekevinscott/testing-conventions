"""End-to-end tests for the wiring-detect-action helper: real file reads, no mocks.

Runs the script's `__main__` entry point in-process via `runpy` against temp workflow files,
covering `main`, the file read, and the `__main__` guard (the lines the pure unit suite does not
reach). A workflow that uses the detect action exits 0; one that does not exits 1. `argv[0]` is a
bogus path so a mutant that reads it instead of `argv[1]` fails on a missing file.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_wiring_detect_action.py"

WIRED = "jobs:\n  detect:\n    steps:\n      - uses: ./.github/actions/detect@v0\n"
UNWIRED = "jobs:\n  detect:\n    steps:\n      - run: echo inline scan\n"


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
    wf.write_text(WIRED)
    assert run([str(tmp_path / "prog-not-a-file"), str(wf)]) == 0
    assert "reusable workflow detects via the detect action" in capsys.readouterr().out


def test_e2e_fails_on_an_unwired_workflow(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(UNWIRED)
    assert run([str(tmp_path / "prog-not-a-file"), str(wf)]) == 1
    assert "::error::" in capsys.readouterr().out
