"""End-to-end tests for the hermetic-wired command: the real workflow files, click's CliRunner.

The mutation tests drop the `CLI_COMMAND` env line from one real step at a time, so each asserts
the check against the regression itself rather than against a hand-written fixture.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.hermetic_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "testing-conventions.yml"
CALLERS = (
    str(REPO_ROOT / ".github" / "workflows" / "testing-conventions-selftest.yml"),
    str(REPO_ROOT / ".github" / "workflows" / "dogfood.yml"),
)
ENV_LINE = "          CLI_COMMAND: ${{ needs.detect.outputs.cli_command }}\n"


def drop_nth_env_line(text, index):
    """`text` with the `index`-th `CLI_COMMAND` env line removed, un-wiring exactly that step."""
    pieces = text.split(ENV_LINE)
    joined = pieces[0]
    for position, piece in enumerate(pieces[1:]):
        joined += ("" if position == index else ENV_LINE) + piece
    return joined


def run_against(tmp_path, text):
    mutated = tmp_path / "testing-conventions.yml"
    mutated.write_text(text)
    return CliRunner().invoke(cli, [str(mutated), *CALLERS])


def test_the_real_workflow_wires_cli_command_into_every_fallback_step():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "derived, caller-built, and fully wired" in result.output


def test_every_step_running_the_fallback_carries_its_own_env_line():
    text = WORKFLOW.read_text()
    assert text.count("${CLI_COMMAND:-") == text.count(ENV_LINE)


def test_dropping_any_single_real_step_env_line_fails_the_check(tmp_path):
    text = WORKFLOW.read_text()
    total = text.count(ENV_LINE)
    assert total > 1
    for index in range(total):
        mutated = drop_nth_env_line(text, index)
        assert mutated.count(ENV_LINE) == total - 1
        assert "${CLI_COMMAND:-" in mutated
        result = run_against(tmp_path, mutated)
        assert result.exit_code == 1, f"step {index} un-wired but the check passed"
        assert "::error::" in result.output


def test_the_unmutated_copy_passes_so_the_mutation_is_what_reds_it(tmp_path):
    assert run_against(tmp_path, WORKFLOW.read_text()).exit_code == 0
