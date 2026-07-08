"""End-to-end tests for the detect-package-root-py command: click's CliRunner.

`CliRunner` invokes the real command with the detect outputs as positional arguments — the way the
self-test's `run:` step passes them — and captures the exit code and output. A passing fixture
exits 0 and echoes the outputs; a wrong output exits 1 with a GitHub Actions `::error::`
annotation.
"""
from click.testing import CliRunner

from checks.detect_package_root_py.cli import cli

GOOD_ARGS = [
    ".github/selftest/monorepo/packages/py",
    "uv",
    ".github/selftest/monorepo/packages/py/testing-conventions.toml",
]


def test_passes_on_the_expected_outputs():
    result = CliRunner().invoke(cli, GOOD_ARGS)
    assert result.exit_code == 0
    assert "package_root=.github/selftest/monorepo/packages/py" in result.output
    assert "python_env=uv" in result.output


def test_fails_on_a_wrong_output():
    bad = list(GOOD_ARGS)
    bad[1] = "poetry"  # wrong python_env
    result = CliRunner().invoke(cli, bad)
    assert result.exit_code == 1
    assert "::error::expected python_env=uv" in result.output


def test_fails_on_missing_arguments():
    result = CliRunner().invoke(cli, GOOD_ARGS[:1])
    assert result.exit_code != 0
