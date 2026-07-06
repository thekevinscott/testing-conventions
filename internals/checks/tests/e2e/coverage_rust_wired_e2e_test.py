"""End-to-end tests for the coverage-rust-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.coverage_rust_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED = "jobs:\n  coverage:\n    language: ${{ needs.detect.outputs.coverage_languages }}\n"
UNWIRED = "jobs:\n  coverage:\n    language: '[\"python\"]'\n"


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "fans unit-coverage over rust" in result.output


def test_fails_on_a_broken_fixture(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(UNWIRED)
    result = CliRunner().invoke(cli, [str(bad)])
    assert result.exit_code == 1
    assert "::error::" in result.output


def test_default_path_passes_against_the_real_workflow():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "fans unit-coverage over rust" in result.output
