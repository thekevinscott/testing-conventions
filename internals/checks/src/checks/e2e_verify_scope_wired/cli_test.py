"""Colocated unit tests for the e2e-verify-scope-wired command (isolation — no `CliRunner`).

The wiring decision is pinned in `find_missing_wiring_test.py`; here the wired and unwired
fixtures cover the command's echo and raise branches, driven through `.callback` so no
`click.testing` collaborator is imported.
"""
from checks.e2e_verify_scope_wired.cli import REUSABLE_WORKFLOW, cli

WIRED = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
      EXTRA_SCOPE: ${{ needs.detect.outputs.e2e_extra_scope }}
      EXCLUDE: ${{ needs.detect.outputs.e2e_exclude }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE" $EXTRA_SCOPE $EXCLUDE

  packaging:
    name: Packaging
"""

UNWIRED = """\
  e2e-verify:
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT"

  packaging:
    name: Packaging
"""


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_command_echoes_on_a_fully_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "appends detect's extra-scope/exclude roots" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "#294" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
