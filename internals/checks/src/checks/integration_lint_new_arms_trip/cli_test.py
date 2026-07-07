"""Colocated unit tests for the integration-lint-new-arms-trip check (isolation — no CliRunner, no subprocess mocks).

Imports only the unit under test: this check's own `CHECKS` data and `cli` command. The shared
orchestration and exit-code decision are covered once beside `run_checks` / `failure_reason`.
Here we pin this check's `CHECKS` and drive `cli` through its `.callback` over the benign
`true`/`false` builtins (the trailing-command seam), asserting the raise path against the
propagated exception's `.message` — so no `click.testing` or `CheckFailed` is imported.
"""
from checks.integration_lint_new_arms_trip.cli import CHECKS, cli


def test_checks_are_the_expected_invocations():
    assert CHECKS == [
    (
        ["npx", "-y", "testing-conventions", "integration", "lint", "--language", "rust", ".github/selftest/integration-rust/red"],
        True,
        "red Rust crate trips integration lint",
    ),
    (
        ["npx", "-y", "testing-conventions", "integration", "lint", "--language", "typescript", ".github/selftest/integration-typescript/red"],
        True,
        "red TypeScript suite trips integration lint",
    ),
    (
        ["npx", "-y", "testing-conventions", "integration", "lint", "--language", "typescript", ".github/selftest/integration-typescript/clean"],
        False,
        "clean TypeScript suite passes integration lint",
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
