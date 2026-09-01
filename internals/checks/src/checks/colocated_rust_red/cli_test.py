"""Colocated unit tests for the colocated-rust-red check (isolation — no CliRunner, no subprocess mocks).

Imports only the unit under test: this check's own `CHECKS` data and `cli` command. The shared
orchestration and exit-code decision are covered once beside `run_checks` / `failure_reason`.
Here we pin this check's `CHECKS` and drive `cli` through its `.callback` over the benign
`true`/`false` builtins (the trailing-command seam), asserting the raise path against the
propagated exception's `.message` — so no `click.testing` or `CheckFailed` is imported.
"""
from checks.colocated_rust_red.cli import CHECKS, cli


def test_checks_are_the_expected_invocations():
    assert CHECKS == [
    (
        ["./hermetic-cli/testing-conventions", "unit", "colocated-test", "--language", "rust", ".github/selftest/colocated-rust/red"],
        True,
        "orphan crate trips the rust colocated-test arm",
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
