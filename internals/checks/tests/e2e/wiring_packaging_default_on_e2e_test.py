"""End-to-end tests for the wiring-packaging-default-on command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output — no `sys.argv` juggling,
no mocks.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.wiring_packaging_default_on.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED = "  packaging:\n    if: needs.detect.outputs.packaging_dist == 'true'\n"
UNWIRED = "  packaging:\n    if: always()\n"


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "packaging is default-on" in result.output


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
    assert "packaging is default-on" in result.output
