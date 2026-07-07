"""Colocated unit tests for the mutation-package-root-wired command (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. Only the unit under test is imported — the raise path is
asserted against the propagated exception's `.message` rather than importing `CheckFailed`. The
command inlines the single-block check: the wired fixture puts the reference inside the mutation
job, and the neighbour fixture puts it only in the surrounding jobs (before `mutation:` and inside
the next `integration-lint:` job), so the block-boundary — start included, next job excluded — is
what makes the difference. The shared `extract_job_block` is exercised through this real call, and
has its own colocated test.
"""
from checks.mutation_package_root_wired.cli import REUSABLE_WORKFLOW, cli

WIRED = """\
jobs:
  coverage-changed:
    steps:
      - run: uv sync
  mutation:
    steps:
      - run: npm ci --prefix ${{ needs.detect.outputs.package_root }}
  integration-lint:
    steps:
      - run: echo done
"""

# The reference sits only in *neighbouring* jobs — the one before the mutation block and the one
# after it — never inside the mutation job itself.
UNWIRED_NEIGHBOUR = """\
jobs:
  coverage-changed:
    steps:
      - run: uv sync --directory ${{ needs.detect.outputs.package_root }}
  mutation:
    steps:
      - run: npm ci
  integration-lint:
    steps:
      - run: echo ${{ needs.detect.outputs.package_root }}
"""


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_command_echoes_when_the_reference_is_inside_the_mutation_block(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "installs and builds from the derived package root" in capsys.readouterr().out


def test_command_raises_when_only_a_neighbouring_job_is_wired(tmp_path):
    # Block extraction must confine the search to the mutation job: a reference in
    # coverage-changed or integration-lint must not satisfy the check.
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED_NEIGHBOUR)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "the mutation job does not reference" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
