"""Colocated unit tests for the coverage-package-root-wired command (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path is
asserted against the propagated exception's `.message` rather than importing `CheckFailed`; the
pure decision has its own `decide_test.py`.
"""
from checks.coverage_package_root_wired.cli import REUSABLE_WORKFLOW, cli

WIRED = """\
jobs:
  unit-coverage:
    steps:
      - run: npm ci --prefix ${{ needs.detect.outputs.package_root }}
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: echo done
"""

UNWIRED = """\
jobs:
  unit-coverage:
    steps:
      - run: npm ci
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: echo done
"""


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_command_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "install/build at the derived package root" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "does not reference" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
