"""End-to-end tests for the mutation-package-root-wired helper: real file I/O, no mocks.

Runs the script's `__main__` entry in-process via `runpy` against temp workflow fixtures, so
the file-read, `main`, and the `__main__` guard stay on the measured-coverage path.
"""
import os
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_mutation_package_root_wired.py"

WIRED = """\
jobs:
  mutation:
    steps:
      - run: npm ci --prefix ${{ needs.detect.outputs.package_root }}
  integration-lint:
    steps:
      - run: echo done
"""

# Wiring present only in a neighbouring job, not inside the mutation block.
BROKEN = """\
jobs:
  mutation:
    steps:
      - run: npm ci
  integration-lint:
    steps:
      - run: echo ${{ needs.detect.outputs.package_root }}
"""


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
    assert run(["check", str(wf)]) == 0
    assert "mutation job installs and builds from the derived package root" in capsys.readouterr().out


def test_e2e_fails_when_only_a_neighbouring_job_is_wired(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(BROKEN)
    assert run(["check", str(wf)]) == 1
    assert "::error::the mutation job does not reference" in capsys.readouterr().out


def test_e2e_uses_default_workflow_path_when_no_arg(tmp_path, capsys):
    root = tmp_path / ".github" / "workflows"
    root.mkdir(parents=True)
    (root / "testing-conventions.yml").write_text(WIRED)
    origin = os.getcwd()
    os.chdir(tmp_path)
    try:
        assert run(["check"]) == 0
    finally:
        os.chdir(origin)
    assert "installs and builds from the derived package root" in capsys.readouterr().out
