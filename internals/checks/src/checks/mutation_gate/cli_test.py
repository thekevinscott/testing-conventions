"""Colocated unit tests for the mutation-gate check."""
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
