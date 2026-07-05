"""End-to-end tests for the detect-package-root-ts check: run `__main__` in-process.

Running the real entry point via `runpy` keeps `main` and the `__main__` guard on the measured
coverage path. Each test sets `sys.argv` the way the workflow's `run:` invocation does — the
detect outputs as positional arguments — and asserts the exit code and captured stdout.
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


def test_e2e_fails_on_wrong_argument_count(capsys):
    assert run(["check_detect_package_root_ts.py", "only-one"]) == 1
    assert "::error::usage:" in capsys.readouterr().out
