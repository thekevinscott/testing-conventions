"""Colocated unit tests for the packaging-build-wired command (isolation — no `CliRunner`).

The wiring decision is pinned in `find_missing_wiring_test.py`; here the wired and unwired
fixtures cover the command's echo and raise branches, driven through `.callback` so no
`click.testing` collaborator is imported.
"""
from checks.packaging_build_wired.cli import REUSABLE_WORKFLOW, cli

WIRED = """\
  packaging:
    needs: detect
    if: ${{ inputs.packaging_artifact != '' || needs.detect.outputs.packaging_build != '' || needs.detect.outputs.packaging_dist == 'true' }}
    steps:
      - if: ${{ needs.detect.outputs.packaging_language == 'python' }}
        uses: astral-sh/setup-uv@v7
      - env:
          PACKAGING_BUILD: ${{ needs.detect.outputs.packaging_build }}
        run: |
          eval "$PACKAGING_BUILD"
      - run: check rust "$pkg/target/package"/**/*.crate
"""

UNWIRED = WIRED.replace('eval "$PACKAGING_BUILD"', "true")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_command_echoes_on_a_fully_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "scans target/package" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "#335" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
