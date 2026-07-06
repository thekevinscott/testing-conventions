"""Colocated unit tests for the isolation-red check (isolation — no CliRunner, no subprocess mocks).

The pure `failure_reason` and `checks_for` are driven directly; `cli` is driven through its
`.callback` over the benign `true`/`false` builtins, so no `click.testing` or `CheckFailed`
collaborator is imported — the raise path is asserted against the propagated exception's `.message`.
"""
from checks.isolation_red.cli import CHECKS, checks_for, cli, failure_reason


def test_failure_reason_red_check_flags_a_zero_exit():
    # expect_fail=True, a zero exit is the violation.
    assert failure_reason(True, 0) == "the command exited 0, but a non-zero (failing) exit was required"


def test_failure_reason_red_check_passes_a_nonzero_exit():
    # Any truthy (non-zero) exit clears a red-path check — including a signal-kill (negative).
    assert failure_reason(True, 1) is None
    assert failure_reason(True, -9) is None


def test_failure_reason_clean_check_passes_a_zero_exit():
    assert failure_reason(False, 0) is None


def test_failure_reason_clean_check_flags_a_nonzero_exit():
    assert failure_reason(False, 3) == "the command exited 3, but a zero (passing) exit was required"
    assert failure_reason(False, -9) == "the command exited -9, but a zero (passing) exit was required"


def test_checks_for_returns_the_hardcoded_checks_with_no_command():
    assert checks_for([]) is CHECKS


def test_checks_for_treats_a_trailing_command_as_a_red_path_check():
    assert checks_for(("some-cmd",)) == [(["some-cmd"], True, "cli")]
    assert checks_for(("npx", "run")) == [(["npx", "run"], True, "cli")]


def test_checks_are_the_expected_invocations():
    assert CHECKS == [
    (
        ["npx", "-y", "testing-conventions", "unit", "lint", "--language", "rust", ".github/selftest/isolation/rust-red"],
        True,
        "un-isolated crate trips unit lint",
    ),
    ]


def test_declares_a_variadic_command_argument():
    # Assert click's own registered metadata — `.callback` bypasses arg parsing, so this is what
    # pins the `@click.argument` decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "command"
    assert argument.nargs == -1


def test_command_echoes_ok_when_the_red_check_holds(capsys):
    # `false` exits non-zero, so the red-path expectation holds -> "[cli] ok", no raise.
    cli.callback(command=("false",))
    assert "[cli] ok" in capsys.readouterr().out


def test_command_raises_when_the_red_check_is_violated():
    # `true` exits 0, so the red-path expectation is violated -> the check raises.
    try:
        cli.callback(command=("true",))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "[cli]" in error.message
    else:
        raise AssertionError("a passing command must violate the red-path check and raise")
