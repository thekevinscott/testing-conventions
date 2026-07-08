"""End-to-end tests for the mutation-gate check: the real entry point, click's CliRunner, no mocks.

`cli` runs a subprocess per check, so it runs here (the package-root e2e suite), not the isolated
unit suite. A trailing benign command stands in for the real hermetic-CLI invocation: `false` (exit 1) makes
the red-path check hold; `true` (exit 0) violates it. CliRunner captures the exit code and output.
"""
from click.testing import CliRunner

from checks.mutation_gate.cli import cli


def test_red_check_passes_when_the_command_fails():
    result = CliRunner().invoke(cli, ["false"])
    assert result.exit_code == 0
    assert "[cli] ok" in result.output


def test_red_check_fails_when_the_command_passes():
    result = CliRunner().invoke(cli, ["true"])
    assert result.exit_code == 1
    assert "::error::[cli]" in result.output
