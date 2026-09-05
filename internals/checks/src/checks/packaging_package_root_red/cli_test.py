"""Colocated unit tests for the packaging-package-root-red check."""
from checks.packaging_package_root_red.cli import CHECKS, cli


def test_checks_are_the_expected_invocations():
    assert CHECKS == [
    (
        ["./hermetic-cli/testing-conventions", "packaging", ".github/selftest/packaging-package-root/red/dist/packaging-package-root-fixture-0.0.0.tgz", "--language", "typescript"],
        True,
        "red per-package dist tarball trips the packaging check",
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
