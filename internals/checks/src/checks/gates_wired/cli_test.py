"""Colocated unit tests for the gates-wired check."""
from checks.gates_wired.cli import REUSABLE_WORKFLOW, cli


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("if: contains(fromJSON(inputs.gates), 'colocated-test')\n")
    cli.callback(workflow=str(workflow))
    assert "scopes its gates to the `gates` allowlist" in capsys.readouterr().out


def test_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("if: always()\n")
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "gates" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
