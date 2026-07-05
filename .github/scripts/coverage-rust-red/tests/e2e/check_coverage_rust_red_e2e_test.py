"""End-to-end tests for the coverage-rust-red check: the real entry point, no mocks.

Per the standard, an e2e test runs with no mocks. It drives the script's `__main__` entry in-process
via `runpy` — with a single `<mode> <command...>` supplied as `sys.argv`, using the benign shell
builtins `true` (exit 0) and `false` (exit 1) in place of the real npx invocation — so the whole
run -> decide -> report path, the real `run_command` subprocess boundary, and the `__main__` guard
are all measured. `sys.argv` is set and restored around each run, confined to the helper.
"""
import runpy
import sys
from pathlib import Path

SCRIPT = Path(__file__).resolve().parents[2] / "check_coverage_rust_red.py"


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


def test_e2e_red_check_passes_when_the_command_fails(capsys):
    # mode `fail`, a command that exits non-zero -> the assertion holds -> exit 0.
    assert run(["check_coverage_rust_red.py", "fail", "false"]) == 0
    assert "[cli] ok" in capsys.readouterr().out


def test_e2e_red_check_fails_when_the_command_passes(capsys):
    # mode `fail`, a command that exits 0 -> the assertion is violated -> exit 1.
    assert run(["check_coverage_rust_red.py", "fail", "true"]) == 1
    assert "::error::[cli]" in capsys.readouterr().out


def test_e2e_clean_check_passes_when_the_command_passes(capsys):
    assert run(["check_coverage_rust_red.py", "pass", "true"]) == 0
    assert "[cli] ok" in capsys.readouterr().out


def test_e2e_clean_check_fails_when_the_command_fails(capsys):
    assert run(["check_coverage_rust_red.py", "pass", "false"]) == 1
    assert "::error::[cli]" in capsys.readouterr().out
