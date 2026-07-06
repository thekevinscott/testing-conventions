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


def run_named(argv, run_name):
    """Run the script under a caller-chosen `run_name`, reporting whether the `__main__`
    guard fired: the exit code when `main` ran (guard raised SystemExit), or None when the
    guard was skipped. Pins `if __name__ == "__main__"` against comparison-operator mutants.
    """
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name=run_name)
        return None
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old


def test_e2e_guard_runs_main_for_value_equal_but_distinct_main_name(tmp_path, capsys):
    # A runtime-built "__main__" is equal by value but a distinct object, so the real `==`
    # runs main while an `is` mutant would skip it — asserting main ran kills Eq->Is.
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(WIRED)
    name = "".join(["__", "main", "__"])
    assert run_named(["check", str(wf)], name) == 0
    assert "unit-coverage installs/builds" in capsys.readouterr().out


def test_e2e_guard_skips_main_for_name_sorting_below_main(tmp_path, capsys):
    # "AAAA" sorts below "__main__", so the real `==` skips main while a `<=` mutant would
    # run it — asserting main did not run kills Eq->LtE.
    wf = tmp_path / "testing-conventions.yml"
    wf.write_text(WIRED)
    assert run_named(["check", str(wf)], "AAAA") is None
    assert capsys.readouterr().out == ""


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
