"""Colocated unit tests for the suite-toolchain-pinned-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback`, so no `click.testing` collaborator is
imported; the raise path is asserted against the propagated exception's `.message`.
"""
from checks.suite_toolchain_pinned_wired.cli import REUSABLE_WORKFLOW, cli

PINNED = '          uv pip install "coverage==7.15.2" "pytest==9.1.1" testing-conventions\n'
FLOATING = "          uv pip install coverage pytest testing-conventions\n"


def test_echoes_on_a_pinned_workflow(tmp_path, capsys):
    wf = tmp_path / "wf.yml"
    wf.write_text(PINNED)
    cli.callback(workflow=str(wf))
    assert "pins the consumer suite toolchain" in capsys.readouterr().out


def test_raises_and_names_the_floating_engine(tmp_path):
    wf = tmp_path / "wf.yml"
    wf.write_text(FLOATING)
    try:
        cli.callback(workflow=str(wf))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "coverage" in error.message and "pytest" in error.message
    else:
        raise AssertionError("a floating suite toolchain must raise")


def test_raises_with_a_clear_message_when_engines_are_absent(tmp_path):
    wf = tmp_path / "wf.yml"
    wf.write_text("          uv pip install testing-conventions\n")
    try:
        cli.callback(workflow=str(wf))
    except Exception as error:  # noqa: BLE001
        assert "not pinned at all" in error.message
    else:
        raise AssertionError("a workflow that never pins the engines must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
