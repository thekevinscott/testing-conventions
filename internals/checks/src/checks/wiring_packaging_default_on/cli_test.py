"""Colocated unit tests for the wiring-packaging-default-on check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path
is asserted against the propagated exception's `.message` rather than importing `CheckFailed`.
"""
from checks.wiring_packaging_default_on.cli import REUSABLE_WORKFLOW, cli

WIRED = "  packaging:\n    if: needs.detect.outputs.packaging_dist == 'true'\n"
UNWIRED = "  packaging:\n    if: always()\n"


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "packaging is default-on" in capsys.readouterr().out


def test_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "packaging_dist" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
