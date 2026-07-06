"""End-to-end tests for the rust-toolchain-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output — no `sys.argv` juggling,
no mocks.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.rust_toolchain_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED = "jobs:\n  setup:\n    if: ${{ inputs.rust_toolchain }}\n    uses: dtolnay/rust-toolchain\n"
UNWIRED = "jobs:\n  setup:\n    uses: actions/setup-python@v6\n"


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "provisions a Rust toolchain behind `rust_toolchain`" in result.output


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
    assert "provisions a Rust toolchain behind `rust_toolchain`" in result.output
