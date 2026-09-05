"""Colocated unit tests for the isolation-red check."""
from checks.isolation_red.cli import CHECKS, cli


def test_checks_are_the_expected_invocations():
    assert CHECKS == [
    (
        ["./hermetic-cli/testing-conventions", "unit", "lint", "--language", "rust", ".github/selftest/isolation/rust-red"],
        True,
        "un-isolated crate trips unit lint",
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
