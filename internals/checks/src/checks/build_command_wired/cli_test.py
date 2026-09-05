"""Colocated unit tests for the build-command-wired check."""
from checks.build_command_wired.cli import REUSABLE_WORKFLOW, cli

WIRED = "run: ${{ needs.detect.outputs.build_command }}\n"
UNWIRED = "run: echo nothing here\n"


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "wires a build_command step" in capsys.readouterr().out


def test_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "build_command" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
