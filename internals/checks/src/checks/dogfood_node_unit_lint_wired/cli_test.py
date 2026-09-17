from checks.dogfood_node_unit_lint_wired.cli import DOGFOOD_WORKFLOW, cli


WIRED = """\
jobs:
  unit-lint:
    uses: ./.github/workflows/testing-conventions.yml
    with:
      source: packages/node/src
      gates: '[\"unit-lint\"]'
"""


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "dogfood runs unit lint for packages/node/src" in capsys.readouterr().out


def test_raises_on_a_workflow_without_the_node_unit_lint_call(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED.replace("gates: '[\"unit-lint\"]'", "gates: '[\"colocated-test\"]'"))
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "dogfood does not run unit lint for packages/node/src" in error.message
    else:
        raise AssertionError("a workflow without the node unit-lint call must raise")


def test_declares_the_workflow_argument_defaulting_to_dogfood():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == DOGFOOD_WORKFLOW
