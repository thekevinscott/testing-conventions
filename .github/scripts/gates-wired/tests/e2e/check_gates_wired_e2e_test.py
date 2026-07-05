"""End-to-end tests for the gates-wired helper: real files, no mocks.

Runs the script's `__main__` entry in-process via `runpy` so `main`, the file read, and the
`__main__` guard are on the measured-coverage path. A temp fixture stands in for the workflow
(good and broken), and a no-arg run from the repo root exercises the default-path branch
against the real `.github/workflows/testing-conventions.yml`.
"""
import os
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_gates_wired.py"
REPO_ROOT = Path(__file__).resolve().parents[5]


def run(argv):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name="__main__")
        return 0
    except SystemExit as e:
        return e.code or 0
    finally:
        sys.argv = old


def test_passes_on_a_wired_fixture(tmp_path, capsys):
    good = tmp_path / "wf.yml"
    good.write_text("if: contains(fromJSON(inputs.gates), 'unit-lint')\n")
    assert run(["check", str(good)]) == 0
    assert "gates` allowlist" in capsys.readouterr().out


def test_fails_on_a_broken_fixture(tmp_path, capsys):
    bad = tmp_path / "wf.yml"
    bad.write_text("if: always()\n")
    assert run(["check", str(bad)]) == 1
    assert "::error::" in capsys.readouterr().out


def test_default_path_passes_against_the_real_workflow(capsys):
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        assert run(["check"]) == 0
    finally:
        os.chdir(old)
    assert "gates` allowlist" in capsys.readouterr().out
