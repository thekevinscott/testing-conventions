"""Colocated unit tests for the colocated-rust-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback`, so no `click.testing` collaborator is
imported. Only the unit under test is imported — the raise path is asserted against the
propagated exception's `.message`.
"""
from checks.colocated_rust_wired.cli import REUSABLE_WORKFLOW, cli


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_command_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("language: ${{ needs.detect.outputs.colocated_test_languages }}\n")
    cli.callback(workflow=str(workflow))
    assert "fans colocated-test over rust" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("language: '[\"python\"]'\n")
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "colocated_test_languages" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
