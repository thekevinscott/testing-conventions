"""End-to-end tests for the cli-isolation-wired command: real files, click's CliRunner."""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.cli_isolation_wired.cli import CLI_SPEC, ISOLATED_LAUNCHER, REUSABLE_WORKFLOW, cli, invocations

REPO_ROOT = Path(__file__).resolve().parents[4]

ISOLATED = f"""\
jobs:
  static:
    steps:
      - run: ${{CLI_COMMAND:-{ISOLATED_LAUNCHER} {CLI_SPEC}}} unit lint
"""


def test_an_isolated_workflow_file_passes(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(ISOLATED)
    result = CliRunner().invoke(cli, [str(workflow)])
    assert result.exit_code == 0
    assert "all 1 CLI invocations resolve from a temp prefix" in result.output


def test_a_bare_npx_workflow_file_exits_nonzero(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(ISOLATED.replace(ISOLATED_LAUNCHER, "npx -y"))
    result = CliRunner().invoke(cli, [str(workflow)])
    assert result.exit_code != 0


def test_default_path_passes_against_the_real_workflow():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "CLI invocations resolve from a temp prefix" in result.output


def test_the_real_workflow_carries_every_known_invocation():
    assert len(invocations((REPO_ROOT / REUSABLE_WORKFLOW).read_text())) == 10
