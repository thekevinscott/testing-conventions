"""Colocated unit tests for the e2e-verify-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback`, so no `click.testing` collaborator is
imported. The decision itself is covered in `decide_test.py`; here the wired and unwired fixtures
cover the command's echo and raise branches. The raise path is asserted against the exception's
`.message`.
"""
from checks.e2e_verify_wired.cli import REUSABLE_WORKFLOW, cli

BOTH = "inputs:\n  run_e2e:\n    default: false\nrun: e2e verify\n"
ONLY_INPUT = "inputs:\n  run_e2e:\n    default: false\nrun: unit coverage\n"


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_command_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(BOTH)
    cli.callback(workflow=str(workflow))
    assert "offers opt-in e2e verify" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(ONLY_INPUT)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "e2e verify" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
