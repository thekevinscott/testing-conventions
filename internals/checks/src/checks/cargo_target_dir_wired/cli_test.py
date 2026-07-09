"""Colocated unit tests for the cargo-target-dir-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. The raise path is asserted against the propagated
exception's `.message` rather than importing `CheckFailed`.
"""
from checks.cargo_target_dir_wired.cli import REUSABLE_WORKFLOW, _WIRED_PATH, cli

WIRED = "\n".join([_WIRED_PATH] * 4)


def test_echoes_when_all_four_cache_steps_are_wired(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "key on the derived cargo_target_dir" in capsys.readouterr().out


def test_raises_when_fewer_than_four_cache_steps_are_wired(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("\n".join([_WIRED_PATH] * 3))
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "3/4 wired" in error.message
    else:
        raise AssertionError("fewer than four wired cache steps must raise")


def test_raises_when_a_bare_package_root_path_survives(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED + "\n${{ needs.detect.outputs.package_root }}/target\n")
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "bare package_root path present: True" in error.message
    else:
        raise AssertionError("a surviving bare package_root path must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses arg
    # parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
