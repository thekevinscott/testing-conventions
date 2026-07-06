"""End-to-end tests for the detect-routes-python command: click's CliRunner, no mocks.

The command takes the detect action's `isolation_languages` JSON array as a CLI argument (exactly
as the workflow single-quotes `${{ steps.detect.outputs.isolation_languages }}` into it). `CliRunner`
invokes it and captures the exit code and output. This is a JSON-arg check, so there is no file to
read; the default-argument path stands in for the "default against the real file" case.
"""
from click.testing import CliRunner

from checks.detect_routes_python.cli import cli


def test_passes_when_python_is_routed_in():
    result = CliRunner().invoke(cli, ['["python","rust"]'])
    assert result.exit_code == 0
    assert 'isolation_languages=["python","rust"]' in result.output
    assert "Python routed into the unit-lint matrix" in result.output


def test_fails_when_python_is_absent():
    result = CliRunner().invoke(cli, ['["rust"]'])
    assert result.exit_code == 1
    assert "::error::" in result.output


def test_default_argument_fails_on_an_empty_matrix():
    result = CliRunner().invoke(cli, [])
    assert result.exit_code == 1
    assert "::error::" in result.output
