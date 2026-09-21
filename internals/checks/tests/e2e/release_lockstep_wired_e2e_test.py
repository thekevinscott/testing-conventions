"""End-to-end tests for the release-lockstep-wired command: real files, click's CliRunner.

The command reads the repository's release config, so it runs here (the package-root e2e suite),
not the isolated unit suite.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.release_lockstep_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]

DRIFTED = """
[[package]]
name = "tc-rust"
kind = "crates"
path = "packages/rust"
globs = ["packages/rust/**"]

[[package]]
name = "tc-py"
kind = "pypi"
depends_on = ["tc-rust"]
globs = ["packages/python/**"]
"""


def test_fails_on_a_config_whose_pypi_half_does_not_rebuild_from_the_crate(tmp_path):
    config = tmp_path / "putitoutthere.toml"
    config.write_text(DRIFTED)
    result = CliRunner().invoke(cli, [str(config)])
    assert result.exit_code == 1
    assert "::error::" in result.output
    assert "tc-py" in result.output


def test_default_path_passes_against_the_real_release_config():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "advance together" in result.output
