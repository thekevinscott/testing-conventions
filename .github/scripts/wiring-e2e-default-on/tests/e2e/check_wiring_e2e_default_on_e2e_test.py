"""End-to-end tests for the wiring-e2e-default-on helper: real file reads, no mocks.

Runs the script's `__main__` entry point in-process via `runpy` against temp workflow files,
covering `main`, the file read, and the `__main__` guard. A workflow that gates e2e verify on
`e2e_attestation` exits 0; one that does not exits 1. `argv[0]` is a bogus path so a mutant that
reads it instead of `argv[1]` fails on a missing file.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_wiring_e2e_default_on.py"

WIRED = "  e2e-verify:\n    if: needs.detect.outputs.e2e_attestation == 'true'\n"
UNWIRED = "  e2e-verify:\n    if: always()\n"


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


def test_e2e_passes_when_e2e_verify_gates_on_the_output(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(WIRED)
    assert run([str(tmp_path / "prog-not-a-file"), str(wf)]) == 0
    assert "e2e verify is default-on (gates on e2e_attestation)" in capsys.readouterr().out


def test_e2e_fails_when_e2e_verify_does_not_gate(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(UNWIRED)
    assert run([str(tmp_path / "prog-not-a-file"), str(wf)]) == 1
    assert "::error::" in capsys.readouterr().out
