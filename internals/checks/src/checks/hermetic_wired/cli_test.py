"""Colocated unit tests for the hermetic-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path is
asserted against the propagated exception's `.message`.
"""
from checks.hermetic_wired.cli import GUARD, REUSABLE_WORKFLOW, cli

WIRED = f"""
jobs:
  build-cli:
    if: ${{{{ {GUARD} }}}}
  detect:
    steps:
      - uses: ./.github/actions/detect
    outputs:
      cli_command: x
"""

UNWIRED = "jobs:\n  detect:\n    steps:\n      - uses: thekevinscott/testing-conventions/.github/actions/detect@v0\n"

FLAG_SHAPED = WIRED + "      - run: echo ${{ inputs.hermetic }}\n"


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "hermetic build-from-HEAD mode" in capsys.readouterr().out


def test_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "hermetic" in error.message
        assert "the derivation guard" in error.message
        assert "a `build-cli` job" in error.message
        assert "a local" in error.message
        assert "a `cli_command`" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_raises_with_only_the_missing_pieces_named(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED.replace("cli_command: x", ""))
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001
        assert "a `cli_command`" in error.message
        assert "the derivation guard" not in error.message
    else:
        raise AssertionError("a workflow missing only cli_command must raise")


def test_raises_on_a_flag_shaped_workflow_even_when_fully_wired(tmp_path):
    # `inputs.hermetic` is the rejected design creeping back in; it must fail even when every
    # derivation piece is also present.
    workflow = tmp_path / "wf.yml"
    workflow.write_text(FLAG_SHAPED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001
        assert "inputs.hermetic" in error.message
        assert "never declared by an input" in error.message
    else:
        raise AssertionError("a workflow referencing inputs.hermetic must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
