"""End-to-end tests for the static-gates-wired command: real files, click's CliRunner.

The command reads a workflow file, so it runs here (the package-root e2e suite), not the isolated
unit suite. `CliRunner` invokes it and captures the exit code and output — no `sys.argv` juggling,
no mocks. The default-path test runs against this repo's real reusable workflow, which is what
proves the five static gates are wired there and not merely in a fixture.
"""
import os
from pathlib import Path

from click.testing import CliRunner

from checks.static_gates_wired.cli import cli

REPO_ROOT = Path(__file__).resolve().parents[4]

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
        if: ${{ contains(inputs.gates, '"colocated-test"') }}
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


def test_passes_on_a_wired_fixture(tmp_path):
    good = tmp_path / "wf.yml"
    good.write_text(WIRED)
    result = CliRunner().invoke(cli, [str(good)])
    assert result.exit_code == 0
    assert "run as steps of one `static` job" in result.output


def test_fails_when_the_one_function_step_is_missing(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(WIRED.replace("unit one-function-per-file --language", "REMOVED"))
    result = CliRunner().invoke(cli, [str(bad)])
    assert result.exit_code == 1
    assert "the one-function-per-file gate step is missing" in result.output


def test_fails_when_the_one_function_gate_guard_is_missing(tmp_path):
    bad = tmp_path / "wf.yml"
    bad.write_text(WIRED.replace("contains(inputs.gates, '\"one-function-per-file\"')", "REMOVED"))
    result = CliRunner().invoke(cli, [str(bad)])
    assert result.exit_code == 1
    assert "the one-function-per-file gate-membership guard is missing" in result.output


def test_default_path_passes_against_the_real_workflow():
    old = os.getcwd()
    os.chdir(REPO_ROOT)
    try:
        result = CliRunner().invoke(cli, [])
    finally:
        os.chdir(old)
    assert result.exit_code == 0
    assert "the five static gates run as steps of one `static` job per language" in result.output
