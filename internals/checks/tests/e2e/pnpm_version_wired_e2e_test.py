"""End-to-end tests for the pnpm-version-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. The default-path test is the one that matters: it holds the real reusable workflow to
the rule, so a literal creeping back into any of its four `pnpm/action-setup` steps fails here.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.pnpm_version_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED = "      - uses: pnpm/action-setup@v5\n        with:\n          version: ${{ needs.detect.outputs.ts_pnpm_version }}\n"
LITERAL = '      - uses: pnpm/action-setup@v5\n        with:\n          version: ">=11"\n'


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "take their version from detect" in result.output


def test_fails_on_a_literal_fixture(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(LITERAL)
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
    assert "take their version from detect" in result.output
