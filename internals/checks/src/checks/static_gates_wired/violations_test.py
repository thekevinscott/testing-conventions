"""Colocated unit tests for the static-job consolidation decision (isolation — pure, no I/O).

A fully-wired sample yields no problems; each legacy job header or missing required substring
yields exactly one.
"""
from checks.static_gates_wired.violations import _LEGACY_JOBS, _REQUIRED, violations

# A minimal `static` job carrying every required substring and none of the legacy job headers.
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


def test_violations_empty_on_a_fully_wired_workflow():
    assert violations(WIRED) == []


def test_each_legacy_job_header_is_a_violation():
    for job in _LEGACY_JOBS:
        text = WIRED + f"  {job}:\n    runs-on: ubuntu-latest\n"
        assert f"the legacy `{job}` job still exists" in violations(text)


def test_a_workflow_without_the_one_function_step_is_a_violation():
    text = WIRED.replace("unit one-function-per-file --language", "REMOVED")
    assert "the one-function-per-file gate step is missing" in violations(text)


def test_a_workflow_without_the_one_function_guard_is_a_violation():
    text = WIRED.replace("contains(inputs.gates, '\"one-function-per-file\"')", "REMOVED")
    assert "the one-function-per-file gate-membership guard is missing" in violations(text)


def test_each_missing_required_substring_is_a_violation():
    for needle, message in _REQUIRED:
        text = WIRED.replace(needle, "REMOVED")
        assert message in violations(text)
