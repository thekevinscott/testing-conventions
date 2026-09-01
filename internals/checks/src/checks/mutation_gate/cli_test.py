"""Colocated unit tests for the mutation-gate check (isolation — no CliRunner, no subprocess mocks).

Imports only the unit under test: this check's own `CHECKS` data and `cli` command. The shared
orchestration and exit-code decision are covered once beside `run_checks` / `failure_reason`.
Here we pin this check's `CHECKS` and drive `cli` through its `.callback` over the benign
`true`/`false` builtins (the trailing-command seam), asserting the raise path against the
propagated exception's `.message` — so no `click.testing` or `CheckFailed` is imported.
"""
from checks.mutation_gate.cli import CHECKS, cli


def test_checks_are_the_expected_invocations():
    assert CHECKS == [
    (
        ["./hermetic-cli/testing-conventions", "unit", "mutation", "--language", "rust", ".github/selftest/mutation/clean"],
        False,
        "clean crate passes unit mutation",
    ),
    (
        ["./hermetic-cli/testing-conventions", "unit", "mutation", "--language", "rust", ".github/selftest/mutation/survivor"],
        True,
        "survivor crate trips the mutation gate",
    ),
]


def test_declares_a_variadic_command_argument():
    (argument,) = cli.params
    assert argument.name == "command"
    assert argument.nargs == -1


def test_command_echoes_ok_when_the_red_check_holds(capsys):
    cli.callback(command=("false",))
    assert "[cli] ok" in capsys.readouterr().out


def test_command_raises_when_the_red_check_is_violated():
    try:
        cli.callback(command=("true",))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "[cli]" in error.message
    else:
        raise AssertionError("a passing command must violate the red-path check and raise")
