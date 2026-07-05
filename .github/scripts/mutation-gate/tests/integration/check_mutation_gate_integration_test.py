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

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))  # .github/scripts/mutation-gate
import check_mutation_gate as m  # noqa: E402


@pytest.fixture
def command():
    """Patch the subprocess boundary and yield the mock for per-test exit-code setup."""
    with patch.object(m, "run_command") as run_command:
        yield run_command


def _returns(*codes):
    """A side effect yielding a CompletedProcess-like result per call, one per given exit code."""
    return [SimpleNamespace(returncode=code) for code in codes]


def test_main_passes_when_both_checks_meet_their_expectation(command, capsys):
    # CHECK 0 is a clean-path check (wants exit 0); CHECK 1 is a red-path check (wants non-zero).
    command.side_effect = _returns(0, 1)
    assert m.main(["check_mutation_gate.py"]) == 0
    out = capsys.readouterr().out
    assert "[clean crate passes unit mutation] ok" in out
    assert "[survivor crate trips the mutation gate] ok" in out
    assert "::error::" not in out
    assert command.call_count == 2


def test_main_fails_when_the_clean_check_unexpectedly_fails(command, capsys):
    command.side_effect = _returns(1, 1)
    assert m.main(["check_mutation_gate.py"]) == 1
    assert "::error::[clean crate passes unit mutation]" in capsys.readouterr().out


def test_main_fails_when_the_red_check_unexpectedly_passes(command, capsys):
    command.side_effect = _returns(0, 0)
    assert m.main(["check_mutation_gate.py"]) == 1
    assert "::error::[survivor crate trips the mutation gate]" in capsys.readouterr().out


def test_main_runs_the_two_hardcoded_commands(command):
    command.side_effect = _returns(0, 1)
    m.main(["check_mutation_gate.py"])
    ran = [call.args[0] for call in command.call_args_list]
    assert ran == [command_ for command_, _expect, _label in m.CHECKS]
