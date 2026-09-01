"""Colocated unit tests for the cli-isolation-wired check (isolation — no `CliRunner`)."""
from checks.cli_isolation_wired.cli import (
    CLI_SPEC,
    ISOLATED_LAUNCHER,
    REUSABLE_WORKFLOW,
    cli,
    invocations,
    violations,
)

ISOLATED = f"""\
jobs:
  static:
    steps:
      - name: Check colocated test
        run: ${{CLI_COMMAND:-{ISOLATED_LAUNCHER} {CLI_SPEC}}} unit colocated-test
      - name: Lint
        run: ${{CLI_COMMAND:-{ISOLATED_LAUNCHER} {CLI_SPEC}}} unit lint
"""

BARE_NPX = ISOLATED.replace(ISOLATED_LAUNCHER, "npx -y", 1)


def test_every_isolated_invocation_is_no_violation():
    assert violations(ISOLATED) == []


def test_a_bare_npx_invocation_is_a_violation():
    problems = violations(BARE_NPX)
    assert len(problems) == 1
    assert problems[0].startswith("line 5: ")


def test_a_violation_quotes_the_offending_line():
    assert "npx -y" in violations(BARE_NPX)[0]


def test_invocations_finds_every_line_naming_the_spec():
    assert [number for number, _ in invocations(ISOLATED)] == [5, 7]


def test_a_workflow_naming_no_spec_yields_no_invocations():
    assert invocations("jobs:\n  detect:\n    steps: []\n") == []


def test_echoes_the_count_on_an_isolated_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(ISOLATED)
    cli.callback(workflow=str(workflow))
    assert "all 2 CLI invocations resolve from a temp prefix" in capsys.readouterr().out


def test_raises_naming_every_offending_line(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(BARE_NPX)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "a consumer's devDependency copy shadows the intended release" in error.message
        assert "line 5: " in error.message
    else:
        raise AssertionError("a bare-npx workflow must raise")


def test_a_workflow_matching_no_spec_fails_rather_than_passing_vacuously(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(ISOLATED.replace(CLI_SPEC, "testing-conventions"))
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "inspected nothing and would pass whatever the launcher is" in error.message
    else:
        raise AssertionError("a workflow naming no spec must raise")


def test_declares_the_workflow_argument_with_its_default():
    (workflow,) = cli.params
    assert workflow.name == "workflow"
    assert workflow.default == REUSABLE_WORKFLOW
