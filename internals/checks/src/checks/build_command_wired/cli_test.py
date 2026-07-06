"""Colocated unit tests for the build-command-wired check (isolation — no `CliRunner`).

The pure predicate is driven directly; the `cli` command is driven through its `.callback`
(the undecorated function), so no `click.testing` collaborator is imported. Only the unit under
test is imported — a first-party collaborator (e.g. `CheckFailed`) would be flagged, so the
raise path is asserted against the propagated exception's `.message` instead.
"""
from checks.build_command_wired.cli import DEFAULT_WORKFLOW, cli, wires_build_command


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == DEFAULT_WORKFLOW


def test_true_when_the_build_command_output_is_referenced():
    assert wires_build_command("run: ${{ needs.detect.outputs.build_command }}") is True


def test_false_when_the_output_is_absent():
    assert wires_build_command("run: echo no build step here") is False


def test_false_on_empty_text():
    assert wires_build_command("") is False


def test_command_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("run: ${{ needs.detect.outputs.build_command }}\n")
    cli.callback(workflow=str(workflow))
    assert "wires a build_command step" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("run: echo nothing here\n")
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "build_command" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
