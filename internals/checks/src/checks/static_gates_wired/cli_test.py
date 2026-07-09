"""Colocated unit tests for the static-gates-wired check (isolation — no `CliRunner`).

The `cli` command is driven through its `.callback` (the undecorated function), so no
`click.testing` collaborator is imported. `violations` — the pure decision — is exercised directly:
a fully-wired sample yields no problems, each legacy job header or missing required substring yields
exactly one. The raise path is asserted against the propagated exception's `.message` rather than
importing `CheckFailed`.
"""
from checks.static_gates_wired.cli import (
    _LEGACY_JOBS,
    _REQUIRED,
    REUSABLE_WORKFLOW,
    cli,
    violations,
)

# A minimal `static` job carrying every substring the check requires, and none of the four legacy
# job headers — the consolidated shape the implementation lands.
WIRED = """\
jobs:
  detect:
    runs-on: ubuntu-latest
  static:
    strategy:
      matrix:
        language: ${{ fromJSON(needs.detect.outputs.static_languages || needs.detect.outputs.integration_lint_languages) }}
    steps:
      - name: Check colocated test
        if: ${{ (inputs.gates == '' || contains(inputs.gates, '"colocated-test"')) }}
        run: tc unit colocated-test --language "$LANGUAGE" --config "$CONFIG" "$SCAN_PATH"
      - name: Check co-change
        if: ${{ !cancelled() && github.event_name == 'pull_request' }}
        run: tc unit colocated-test --language "$LANGUAGE" --base "$BASE" --config "$CONFIG" "$SCAN_PATH"
      - name: Check unit lint
        if: ${{ !cancelled() && contains(inputs.gates, '"unit-lint"') }}
        run: tc unit lint --language "$LANGUAGE" --config "$CONFIG" "$SCAN_PATH"
      - name: Lint integration tests
        if: ${{ !cancelled() && contains(inputs.gates, '"integration-lint"') }}
        run: tc integration lint --language "$LANGUAGE" --config "$CONFIG" "$SCAN_PATH"
"""


def test_violations_empty_on_a_fully_wired_workflow():
    assert violations(WIRED) == []


def test_echoes_on_a_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "run as steps of one `static` job" in capsys.readouterr().out


def test_raises_listing_every_problem_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text("jobs:\n  colocated-test:\n    runs-on: ubuntu-latest\n")
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "consolidate the four static gates" in error.message
        assert "the legacy `colocated-test` job still exists" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_each_legacy_job_header_is_a_violation():
    for job in _LEGACY_JOBS:
        text = WIRED + f"  {job}:\n    runs-on: ubuntu-latest\n"
        assert f"the legacy `{job}` job still exists" in violations(text)


def test_each_missing_required_substring_is_a_violation():
    for needle, message in _REQUIRED:
        text = WIRED.replace(needle, "REMOVED")
        assert message in violations(text)


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses arg
    # parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
