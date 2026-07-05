"""End-to-end tests for the coverage-package-root-wired helper: real file I/O, no mocks.

Runs the script's `__main__` entry in-process via `runpy` against temp workflow fixtures, so
the file-read, `main`, and the `__main__` guard stay on the measured-coverage path. A passing
fixture wires both coverage jobs; a broken one drops the reference from one job.
"""
import os
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_coverage_package_root_wired.py"

WIRED = """\
jobs:
  unit-coverage:
    steps:
      - run: npm ci --prefix ${{ needs.detect.outputs.package_root }}
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: echo done
"""

BROKEN = """\
jobs:
  unit-coverage:
    steps:
      - run: npm ci
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: echo done
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
    out = capsys.readouterr().out
    assert "unit-coverage installs/builds at needs.detect.outputs.package_root" in out
    assert "coverage-changed installs/builds at needs.detect.outputs.package_root" in out


def test_e2e_fails_when_a_coverage_job_is_unwired(tmp_path, capsys):
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(BROKEN)
    assert run(["check", str(wf)]) == 1
    out = capsys.readouterr().out
    assert "::error::the unit-coverage job doesn't reference" in out


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
    assert "unit-coverage installs/builds" in capsys.readouterr().out
