import os
from pathlib import Path

from click.testing import CliRunner

from checks.dogfood_node_unit_lint_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]


def test_default_path_passes_against_the_real_dogfood_workflow():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "dogfood runs unit lint for packages/node/src" in result.output
