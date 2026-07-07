"""Colocated unit tests for the wiring-detect-action check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path is
asserted against the propagated exception's `.message`.
"""
from checks.wiring_detect_action.cli import REUSABLE_WORKFLOW, cli

WIRED = "      - uses: ./.github/actions/detect@v0\n"
UNWIRED = "      - run: echo inline scan\n"


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "detects via the detect action" in capsys.readouterr().out


def test_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "detect action" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
