"""End-to-end tests for the github-helpers-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output — no `sys.argv` juggling,
no mocks.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.github_helpers_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WIRED = (
    'for rule in "unit colocated-test" "unit lint" "unit coverage" '
    '"integration lint" "unit mutation --base origin/main"; do\n'
    '  npx -y testing-conventions $rule --language python "$dir"\n'
    "done\n"
)
BROKEN = WIRED.replace('"integration lint" ', "")


def test_passes_on_a_fully_wired_fixture(tmp_path):
    good = tmp_path / "dogfood.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "runs all five Python arms" in result.output


def test_fails_on_a_broken_fixture(tmp_path):
    bad = tmp_path / "dogfood.yml"
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
    assert "runs all five Python arms" in result.output
