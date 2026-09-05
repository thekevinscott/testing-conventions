"""Colocated unit tests for the static-gates-wired command (isolation — no `CliRunner`).

The consolidation decision is pinned in `violations_test.py`; here the `cli` command is driven
through its `.callback` (the undecorated function), and the raise path is asserted against the
propagated exception's `.message`.
"""
from checks.static_gates_wired.cli import REUSABLE_WORKFLOW, cli

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
      - name: Check one function per file
        if: ${{ !cancelled() && contains(inputs.gates, '"one-function-per-file"') }}
        run: tc unit one-function-per-file --language "$LANGUAGE" --config "$CONFIG" "$SCAN_PATH"
      - name: Lint integration tests
        if: ${{ !cancelled() && contains(inputs.gates, '"integration-lint"') }}
        run: tc integration lint --language "$LANGUAGE" --config "$CONFIG" "$SCAN_PATH"
"""


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
        assert "consolidate the five static gates" in error.message
        assert "the legacy `colocated-test` job still exists" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW
