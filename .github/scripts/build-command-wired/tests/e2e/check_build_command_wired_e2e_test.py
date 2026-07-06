"""End-to-end tests for the build-command-wired helper: real files, no mocks.

Runs the script's `__main__` entry in-process via `runpy` so `main`, the file read, and the
`__main__` guard are on the measured-coverage path. A temp fixture stands in for the workflow
(good and broken), and a no-arg run from the repo root exercises the default-path branch
against the real `.github/workflows/testing-conventions.yml`. Two guard tests vary the module
`__name__` (via `runpy`'s `run_name`) to pin the `==` in `if __name__ == "__main__"` against
its `is` and `<=` mutants.
"""
import os
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_build_command_wired.py"
REPO_ROOT = Path(__file__).resolve().parents[5]
WIRED = "jobs:\n  build:\n    run: ${{ needs.detect.outputs.build_command }}\n"

# Both equal "__main__" by value, but built at runtime so they are distinct objects (not the
# interned literal in the module) — and one sorts lexicographically below "__main__".
DISTINCT_MAIN = "".join(list("__main__"))
LOWER_THAN_MAIN = "".join(list("__aaa__"))


def run(argv, run_name="__main__"):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name=run_name)
        return 0
    except SystemExit as e:
        return e.code or 0
    finally:
        sys.argv = old


def test_passes_on_a_wired_fixture(tmp_path, capsys):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    assert run(["check", str(good)]) == 0
    assert "wires a build_command step" in capsys.readouterr().out


def test_fails_on_a_broken_fixture(tmp_path, capsys):
    bad = tmp_path / "wf.yml"
    bad.write_text("jobs:\n  build:\n    run: echo nothing here\n")
    assert run(["check", str(bad)]) == 1
    assert "::error::" in capsys.readouterr().out


def test_default_path_passes_against_the_real_workflow(capsys):
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        assert run(["check"]) == 0
    finally:
        os.chdir(old)
    assert "wires a build_command step" in capsys.readouterr().out


def test_guard_runs_main_when_name_equals_main_by_value(tmp_path, capsys):
    # __name__ is "__main__" by value but a distinct object: `==` is True (main runs and
    # prints), while the `is` mutant sees identity False and skips main. Asserting the output
    # is present kills Eq->Is.
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    assert run(["check", str(good)], run_name=DISTINCT_MAIN) == 0
    assert "wires a build_command step" in capsys.readouterr().out


def test_guard_skips_main_for_a_lower_sorting_name(tmp_path, capsys):
    # A name sorting below "__main__" is unequal, so `==` is False and main never runs (no
    # output); the `<=` mutant would run it. Asserting no output kills Eq->LtE.
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    assert run(["check", str(good)], run_name=LOWER_THAN_MAIN) == 0
    assert capsys.readouterr().out == ""
