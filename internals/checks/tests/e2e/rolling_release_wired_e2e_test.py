"""End-to-end tests for the rolling-release-wired command: real files, click's CliRunner.

The command reads two workflow files, so it runs here (the package-root e2e suite), not the
isolated unit suite. `CliRunner` invokes it and captures the exit code and output.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.rolling_release_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
GATED = "on:\n  workflow_run:\n    workflows: [Release]\nconcurrency:\n  group: move-major-tag\n"


def test_passes_on_a_gated_move_tag_and_clean_release(tmp_path):
    move_tag = tmp_path / "move-major-tag.yml"
    move_tag.write_text(GATED)
    release = tmp_path / "release.yml"
    release.write_text("run: npm publish\n")
    result = CliRunner().invoke(cli, [str(move_tag), str(release)])
    assert result.exit_code == 0
    assert "gated move-major-tag workflow" in result.output


def test_fails_when_move_tag_absent_and_release_moves_inline(tmp_path):
    move_tag = tmp_path / "does-not-exist.yml"
    release = tmp_path / "release.yml"
    release.write_text("run: git tag -f v0 $SHA\n")
    result = CliRunner().invoke(cli, [str(move_tag), str(release)])
    assert result.exit_code == 1
    assert "::error::" in result.output


def test_default_paths_pass_against_the_real_workflows():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "gated move-major-tag workflow" in result.output
