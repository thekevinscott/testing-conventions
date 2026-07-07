"""Colocated unit tests for the e2e-verify-checks-out-pr-head check (isolation — no `CliRunner`).

The block-scoping decision is covered in `block_test.py`; here the pinned and unpinned fixtures
cover the command's echo and raise branches, driven through `.callback` so no `click.testing`
collaborator is imported. The raise path is asserted against the exception's `.message`.
"""
from checks.e2e_verify_checks_out_pr_head.cli import REUSABLE_WORKFLOW, cli

PINNED = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "        with:\n"
    "          ref: ${{ github.event.pull_request.head.sha || github.sha }}\n"
    "  packaging:\n"
    "    name: pkg\n"
)
UNPINNED = (
    "  e2e-verify:\n"
    "    steps:\n"
    "      - uses: actions/checkout@v6\n"
    "  packaging:\n"
    "    name: pkg\n"
)


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_command_echoes_on_a_pinned_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(PINNED)
    cli.callback(workflow=str(workflow))
    assert "checks out the PR head commit" in capsys.readouterr().out


def test_command_raises_on_an_unpinned_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNPINNED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "head.sha" in error.message
    else:
        raise AssertionError("an unpinned workflow must raise")
