"""End-to-end tests for the cli-node-engine-wired command: real files, click's CliRunner."""
import json
import os
from pathlib import Path

from click.testing import CliRunner

from checks.cli_node_engine_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]

WIRED = """\
jobs:
  static:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/setup-node@v6
        with:
          node-version: 24
      - name: Check colocated test
        run: ${CLI_COMMAND:-npx -y "testing-conventions${VERSION:+@$VERSION}"} unit colocated-test
"""


def manifest_at(tmp_path, requirement=">=24"):
    path = tmp_path / "package.json"
    path.write_text(json.dumps({"engines": {"node": requirement}}))
    return str(path)


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good), manifest_at(tmp_path)])
    assert result.exit_code == 0
    assert "every CLI-invoking job provisions node 24 or newer" in result.output


def test_fails_when_the_setup_node_step_is_gated_by_a_language(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(
        WIRED.replace(
            "      - uses: actions/setup-node@v6",
            "      - if: matrix.language == 'typescript'\n        uses: actions/setup-node@v6",
        )
    )
    result = CliRunner().invoke(cli, [str(bad), manifest_at(tmp_path)])
    assert result.exit_code == 1
    assert "`static` invokes the CLI with no unconditional `setup-node` step" in result.output


def test_fails_when_the_engines_floor_outruns_the_pinned_node(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good), manifest_at(tmp_path, ">=26")])
    assert result.exit_code == 1
    assert "`static` pins node 24, below the floor of 26" in result.output


def test_default_path_passes_against_the_real_workflow_and_manifest():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "every CLI-invoking job provisions node" in result.output
