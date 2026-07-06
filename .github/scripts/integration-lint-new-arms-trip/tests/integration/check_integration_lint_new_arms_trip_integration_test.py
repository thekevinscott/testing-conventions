"""Integration tests for `main`: the real orchestration over the hardcoded CHECKS, the subprocess
boundary mocked.

Per the standard, an integration test runs first-party code for real and mocks the externals.
`run_command` is the one external here, so the `command` fixture patches it and yields the mock;
each test sets the per-call exit codes and asserts the exit status and printed lines. The patching
lives in the fixture, never inline in a test body.
"""
import sys
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))
import check_integration_lint_new_arms_trip as m  # noqa: E402


@pytest.fixture
def command():
    """Patch the subprocess boundary and yield the mock for per-test exit-code setup."""
    with patch.object(m, "run_command") as run_command:
        yield run_command


def _returns(*codes):
    """A side effect yielding a CompletedProcess-like result per call, one per given exit code."""
    return [SimpleNamespace(returncode=code) for code in codes]


def test_main_passes_when_every_check_meets_its_expectation(command, capsys):
    command.side_effect = _returns(1, 1, 0)
    assert m.main(["check_integration_lint_new_arms_trip.py"]) == 0
    out = capsys.readouterr().out
    assert "::error::" not in out
    assert "[red Rust crate trips integration lint] ok" in out
    assert "[red TypeScript suite trips integration lint] ok" in out
    assert "[clean TypeScript suite passes integration lint] ok" in out
    assert command.call_count == 3


def test_main_fails_when_check_0_is_violated(command, capsys):
    command.side_effect = _returns(0, 1, 0)
    assert m.main(["check_integration_lint_new_arms_trip.py"]) == 1
    assert "::error::[red Rust crate trips integration lint]" in capsys.readouterr().out


def test_main_fails_when_check_1_is_violated(command, capsys):
    command.side_effect = _returns(1, 0, 0)
    assert m.main(["check_integration_lint_new_arms_trip.py"]) == 1
    assert "::error::[red TypeScript suite trips integration lint]" in capsys.readouterr().out


def test_main_fails_when_check_2_is_violated(command, capsys):
    command.side_effect = _returns(1, 1, 1)
    assert m.main(["check_integration_lint_new_arms_trip.py"]) == 1
    assert "::error::[clean TypeScript suite passes integration lint]" in capsys.readouterr().out


def test_main_runs_the_hardcoded_commands(command):
    command.side_effect = _returns(1, 1, 0)
    m.main(["check_integration_lint_new_arms_trip.py"])
    ran = [call.args[0] for call in command.call_args_list]
    assert ran == [command for command, _expect, _label in m.CHECKS]
