"""Colocated unit tests for the detect-package-root-py command (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path is
asserted against the propagated exception's `.message`. The pure decision is tested in
`evaluate_test.py`; each declared `@click.argument` is pinned via `cli.params`.
"""
from checks.detect_package_root_py.cli import cli

GOOD = (
    ".github/selftest/monorepo/packages/py",
    "uv",
    ".github/selftest/monorepo/packages/py/testing-conventions.toml",
)


def test_declares_the_three_detect_output_arguments():
    names = [p.name for p in cli.params]
    assert names == ["package_root", "python_env", "config"]
    assert all(p.required for p in cli.params)


def test_echoes_the_outputs_on_the_expected_fixture(capsys):
    cli.callback(*GOOD)
    out = capsys.readouterr().out
    assert "package_root=.github/selftest/monorepo/packages/py" in out
    assert "python_env=uv" in out


def test_raises_on_a_wrong_output():
    try:
        cli.callback(GOOD[0], "pip", GOOD[2])
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing
        assert "python_env=uv" in error.message
    else:
        raise AssertionError("a wrong detect output must raise")
