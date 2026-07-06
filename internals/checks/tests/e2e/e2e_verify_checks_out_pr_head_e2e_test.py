"""End-to-end tests for the e2e-verify-checks-out-pr-head command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.e2e_verify_checks_out_pr_head.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "        with:\n"
    "          ref: ${{ github.event.pull_request.head.sha || github.sha }}\n"
    "  packaging:\n"
)
BROKEN = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "  packaging:\n"
)


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "checks out the PR head commit" in result.output


def test_fails_on_a_broken_fixture(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(BROKEN)
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
    assert "checks out the PR head commit" in result.output
