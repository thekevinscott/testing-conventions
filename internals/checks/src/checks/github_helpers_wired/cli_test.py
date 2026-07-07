"""Colocated unit tests for the github-helpers-wired check (isolation — no `CliRunner`).

The pure predicate is driven directly; the `cli` command is driven through its `.callback`
(the undecorated function), so no `click.testing` collaborator is imported. Only the unit under
test is imported — a first-party collaborator (e.g. `CheckFailed`) would be flagged, so the
raise path is asserted against the propagated exception's `.message` instead.
"""
from checks.github_helpers_wired.cli import DOGFOOD_HELPERS_WORKFLOW, cli, wires_github_helpers

# A workflow snippet wiring all five arms, in the diff-scoped `--base` form the gate uses.
WIRED = (
    'for rule in "unit colocated-test" "unit lint" "unit coverage" '
    '"integration lint" "unit mutation --base origin/main"; do\n'
    '  npx -y testing-conventions $rule --language python "$dir"\n'
    "done\n"
)


def test_declares_the_workflow_argument_defaulting_to_the_dogfood_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == DOGFOOD_HELPERS_WORKFLOW


def test_true_when_all_five_arms_are_wired():
    assert wires_github_helpers(WIRED) is True


def test_false_when_colocated_test_is_missing():
    assert wires_github_helpers(WIRED.replace("unit colocated-test", "")) is False


def test_false_when_unit_lint_is_missing():
    assert wires_github_helpers(WIRED.replace("unit lint", "")) is False


def test_false_when_unit_coverage_is_missing():
    assert wires_github_helpers(WIRED.replace("unit coverage", "")) is False


def test_false_when_integration_lint_is_missing():
    assert wires_github_helpers(WIRED.replace("integration lint", "")) is False


def test_false_when_mutation_base_flag_is_missing():
    # A bare `unit mutation` (no `--base`) is the whole-tree form the gate does not use, so the
    # diff-scoped mutation arm must read as missing.
    assert wires_github_helpers(WIRED.replace("unit mutation --base origin/main", "unit mutation")) is False


def test_false_on_empty_text():
    assert wires_github_helpers("") is False


def test_command_echoes_on_a_fully_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "dogfood.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "runs all five Python arms" in capsys.readouterr().out


def test_command_raises_on_a_workflow_missing_an_arm(tmp_path):
    workflow = tmp_path / "dogfood.yml"
    workflow.write_text(WIRED.replace("integration lint", ""))
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "unit mutation --base" in error.message
    else:
        raise AssertionError("a workflow missing an arm must raise")
