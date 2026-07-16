"""End-to-end tests for the engines-locked-wired command: real files, click's CliRunner.

The command scans a directory of workflow files and reads the engine lock, so it runs here (the
package-root e2e suite), not the isolated unit suite. `CliRunner` invokes it and captures the exit
code and output — no `sys.argv` juggling, no mocks.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.engines_locked_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
PINNED = "      - run: uv run --with-requirements .github/uv/engines.txt --no-project pytest\n"
FLOATING = "      - run: uv run --with pytest --no-project pytest\n"


def _staged(tmp_path, body):
    workflows = tmp_path / "workflows"
    workflows.mkdir()
    (workflows / "wf.yml").write_text(body)
    lock = tmp_path / "engines.txt"
    lock.write_text("pytest==9.1.1\n")
    return workflows, lock


def test_passes_on_a_pinned_fixture(tmp_path):
    workflows, lock = _staged(tmp_path, PINNED)
    result = CliRunner().invoke(cli, [str(workflows), "--lock", str(lock)])
    assert result.exit_code == 0
    assert "none floats" in result.output


def test_fails_on_a_floating_fixture(tmp_path):
    workflows, lock = _staged(tmp_path, FLOATING)
    result = CliRunner().invoke(cli, [str(workflows), "--lock", str(lock)])
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
    assert "none floats" in result.output
