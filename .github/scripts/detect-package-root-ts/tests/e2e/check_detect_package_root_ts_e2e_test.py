"""End-to-end tests for the detect-package-root-ts check: run `__main__` in-process.

Running the real entry point via `runpy` keeps `main`, the argument-count guard, and the
`__name__ == "__main__"` guard on the measured coverage path. Each test sets `sys.argv` the way
the workflow's `run:` invocation does — the detect outputs as positional arguments — and asserts
the exit code and captured stdout. Two tests drive the `__name__` guard from a lesser module
name (must stay silent) and from a value-equal-but-distinct `"__main__"` object (must run main),
pinning the guard's `==` against `<=`/`is` mutations; a too-many-arguments test pins the
argument-count comparison against a `<` mutation.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_detect_package_root_ts.py"

GOOD_ARGS = [
    ".github/selftest/monorepo/packages/ts",
    "npm",
    "false",
    ".github/selftest/monorepo/packages/ts/testing-conventions.toml",
]


def run(argv, run_name="__main__"):
    old = sys.argv
    sys.argv = argv
    try:
        runpy.run_path(str(SCRIPT), run_name=run_name)
        return 0
    except SystemExit as exit_:
        return exit_.code or 0
    finally:
        sys.argv = old


def test_e2e_passes_on_the_expected_outputs(capsys):
    assert run(["check_detect_package_root_ts.py", *GOOD_ARGS]) == 0
    out = capsys.readouterr().out
    assert "package_root=.github/selftest/monorepo/packages/ts" in out
    assert "ts_package_manager=npm" in out


def test_e2e_fails_on_a_wrong_output(capsys):
    bad = ["check_detect_package_root_ts.py", *GOOD_ARGS]
    bad[2] = "pnpm"  # wrong ts_package_manager
    assert run(bad) == 1
    assert "::error::expected ts_package_manager=npm" in capsys.readouterr().out


def test_e2e_fails_on_too_few_arguments(capsys):
    assert run(["check_detect_package_root_ts.py", "only-one"]) == 1
    assert "::error::usage:" in capsys.readouterr().out


def test_e2e_fails_on_too_many_arguments(capsys):
    assert run(["check_detect_package_root_ts.py", *GOOD_ARGS, "surplus"]) == 1
    assert "::error::usage:" in capsys.readouterr().out


def test_e2e_guard_does_not_run_main_for_a_lesser_module_name(capsys):
    # "AAAAAA" < "__main__"; the `__name__ == "__main__"` guard stays False, so main never runs.
    assert run(["check_detect_package_root_ts.py", *GOOD_ARGS], run_name="AAAAAA") == 0
    assert capsys.readouterr().out == ""


def test_e2e_guard_runs_main_for_a_distinct_main_object(capsys):
    # A fresh string equal to "__main__" but not identical: the `==` guard enters and main runs.
    distinct = "".join(list("__main__"))
    assert run(["check_detect_package_root_ts.py", *GOOD_ARGS], run_name=distinct) == 0
    assert "package_root=.github/selftest/monorepo/packages/ts" in capsys.readouterr().out
