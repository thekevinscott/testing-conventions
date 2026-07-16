"""End-to-end tests for the suite-toolchain-pinned-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.suite_toolchain_pinned_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
PINNED = '          uv pip install "coverage==7.15.2" "pytest==9.1.1" testing-conventions\n'
FLOATING = "          uv pip install coverage pytest testing-conventions\n"


def test_passes_on_a_pinned_fixture(tmp_path):
    wf = tmp_path / "wf.yml"
    wf.write_text(PINNED)
    result = CliRunner().invoke(cli, [str(wf)])
    assert result.exit_code == 0
    assert "pins the consumer suite toolchain" in result.output


def test_fails_on_a_floating_fixture(tmp_path):
    wf = tmp_path / "wf.yml"
    wf.write_text(FLOATING)
    result = CliRunner().invoke(cli, [str(wf)])
    assert result.exit_code == 1
    assert "::error::" in result.output


def test_default_path_passes_against_the_real_reusable_workflow():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "pins the consumer suite toolchain" in result.output
