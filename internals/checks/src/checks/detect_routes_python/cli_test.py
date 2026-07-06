"""Colocated unit tests for the detect-routes-python command (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path is
asserted against the propagated exception's `.message`. The pure decision is tested in
`routes_test.py`.
"""
from checks.detect_routes_python.cli import DEFAULT_ISOLATION_LANGUAGES, cli


def test_echoes_when_python_is_routed_in(capsys):
    cli.callback(isolation_languages='["python","rust"]')
    out = capsys.readouterr().out
    assert 'isolation_languages=["python","rust"]' in out
    assert "Python routed into the unit-lint matrix" in out


def test_raises_when_python_is_absent():
    try:
        cli.callback(isolation_languages='["rust"]')
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "did not route Python" in error.message
    else:
        raise AssertionError("a value without Python must raise")


def test_declares_the_isolation_languages_argument_with_its_default():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "isolation_languages"
    assert argument.default == DEFAULT_ISOLATION_LANGUAGES
