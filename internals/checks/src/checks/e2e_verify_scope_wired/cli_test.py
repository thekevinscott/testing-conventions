"""Colocated unit tests for the e2e-verify-scope-wired check (isolation — no `CliRunner`).

The pure `find_missing_wiring` / `extract_e2e_verify_block` decisions are driven directly over
crafted wired/unwired blocks; the `cli` command is driven through its `.callback` (the
undecorated function), so no `click.testing` collaborator is imported. Only the unit under test
is imported — a first-party collaborator (e.g. `CheckFailed`) would be flagged, so the raise
path is asserted against the propagated exception's `.message` instead.
"""
from checks.e2e_verify_scope_wired.cli import (
    REUSABLE_WORKFLOW,
    _BASE_ERROR,
    _EXCLUDE_ERROR,
    _EXTRA_SCOPE_ERROR,
    _GATE_ERROR,
    _SCOPE_ERROR,
    cli,
    extract_e2e_verify_block,
    find_missing_wiring,
)

WIRED = """\
  e2e-verify:
    name: E2E attestation freshness (e2e verify)
    if: ${{ github.event_name == 'pull_request' && (inputs.gates == '' || contains(inputs.gates, '"e2e-verify"')) }}
    steps:
      - uses: actions/checkout@v6
      - name: Verify the e2e attestation is current
        env:
          PACKAGE_ROOT: ${{ needs.detect.outputs.package_root }}
          SCAN_PATH: ${{ inputs.source }}
          BASE: ${{ inputs.base }}
          EXTRA_SCOPE: ${{ needs.detect.outputs.e2e_extra_scope }}
          EXCLUDE: ${{ needs.detect.outputs.e2e_exclude }}
        run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE" $EXTRA_SCOPE $EXCLUDE

  packaging:
    name: Packaging
"""

UNWIRED = """\
  e2e-verify:
    name: E2E attestation freshness (e2e verify)
    steps:
      - uses: actions/checkout@v6
      - name: Verify the e2e attestation is current
        env:
          PACKAGE_ROOT: ${{ needs.detect.outputs.package_root }}
        run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT"

  packaging:
    name: Packaging
"""

MISSING_SCOPE_FLAG_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --base "$BASE"

  packaging:
    name: Packaging
"""

MISSING_SCAN_PATH_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: something-else
      BASE: ${{ inputs.base }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE"

  packaging:
    name: Packaging
"""

MISSING_BASE_FLAG_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH"

  packaging:
    name: Packaging
"""

MISSING_BASE_ENV_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: something-else
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE"

  packaging:
    name: Packaging
"""

MISSING_PULL_REQUEST_GATE = """\
  e2e-verify:
    if: ${{ inputs.gates == '' || contains(inputs.gates, '"e2e-verify"') }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE"

  packaging:
    name: Packaging
"""

# #333: scope/base/gate all wired, but the extra-scope/exclude wiring varies. Each keeps
# everything the earlier checks require so the walk reaches the new branches.
MISSING_EXTRA_SCOPE_ARG_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
      EXTRA_SCOPE: ${{ needs.detect.outputs.e2e_extra_scope }}
      EXCLUDE: ${{ needs.detect.outputs.e2e_exclude }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE" $EXCLUDE

  packaging:
    name: Packaging
"""

MISSING_EXTRA_SCOPE_ENV_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
      EXTRA_SCOPE: something-else
      EXCLUDE: ${{ needs.detect.outputs.e2e_exclude }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE" $EXTRA_SCOPE $EXCLUDE

  packaging:
    name: Packaging
"""

MISSING_EXCLUDE_ARG_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
      EXTRA_SCOPE: ${{ needs.detect.outputs.e2e_extra_scope }}
      EXCLUDE: ${{ needs.detect.outputs.e2e_exclude }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE" $EXTRA_SCOPE

  packaging:
    name: Packaging
"""

MISSING_EXCLUDE_ENV_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.source }}
      BASE: ${{ inputs.base }}
      EXTRA_SCOPE: ${{ needs.detect.outputs.e2e_extra_scope }}
      EXCLUDE: something-else
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE" $EXTRA_SCOPE $EXCLUDE

  packaging:
    name: Packaging
"""


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    # Assert click's own registered metadata (the `@click.argument`) — `.callback` bypasses
    # arg parsing, so this is what pins the decorator without a CliRunner collaborator.
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


def test_finds_no_missing_wiring_when_scope_base_gate_and_extra_roots_are_all_present():
    assert find_missing_wiring(WIRED) is None


def test_finds_missing_wiring_when_scope_is_absent():
    assert find_missing_wiring(UNWIRED) is not None


def test_finds_missing_wiring_when_only_the_scope_flag_is_absent():
    assert find_missing_wiring(MISSING_SCOPE_FLAG_ONLY) == _SCOPE_ERROR


def test_finds_missing_wiring_when_only_scan_path_is_unbound():
    assert find_missing_wiring(MISSING_SCAN_PATH_ONLY) == _SCOPE_ERROR


def test_finds_missing_wiring_when_only_the_base_flag_is_absent():
    assert find_missing_wiring(MISSING_BASE_FLAG_ONLY) == _BASE_ERROR


def test_finds_missing_wiring_when_only_the_base_env_is_unbound():
    assert find_missing_wiring(MISSING_BASE_ENV_ONLY) == _BASE_ERROR


def test_finds_missing_wiring_when_the_pull_request_gate_is_absent():
    assert find_missing_wiring(MISSING_PULL_REQUEST_GATE) == _GATE_ERROR


def test_finds_missing_wiring_when_only_the_extra_scope_arg_is_absent():
    assert find_missing_wiring(MISSING_EXTRA_SCOPE_ARG_ONLY) == _EXTRA_SCOPE_ERROR


def test_finds_missing_wiring_when_only_the_extra_scope_env_is_unbound():
    assert find_missing_wiring(MISSING_EXTRA_SCOPE_ENV_ONLY) == _EXTRA_SCOPE_ERROR


def test_finds_missing_wiring_when_only_the_exclude_arg_is_absent():
    assert find_missing_wiring(MISSING_EXCLUDE_ARG_ONLY) == _EXCLUDE_ERROR


def test_finds_missing_wiring_when_only_the_exclude_env_is_unbound():
    assert find_missing_wiring(MISSING_EXCLUDE_ENV_ONLY) == _EXCLUDE_ERROR


def test_extract_e2e_verify_block_stops_before_the_next_job():
    block = extract_e2e_verify_block(WIRED)
    assert "e2e-verify:" in block
    assert "packaging:" not in block


def test_extract_e2e_verify_block_is_empty_when_the_job_is_absent():
    assert extract_e2e_verify_block("  packaging:\n    name: Packaging\n") == ""


def test_extract_e2e_verify_block_runs_to_end_when_no_next_job_follows():
    # The e2e-verify job with no following `  packaging:` — the block extends to the end of the
    # file (the `else len(workflow_text)` arm of the ternary).
    text = '  e2e-verify:\n    run: something --scope x --base y $EXTRA_SCOPE $EXCLUDE\n'
    assert extract_e2e_verify_block(text) == text


def test_rejects_a_scope_flag_outside_the_e2e_verify_block():
    # Regression for the actual #294 wiring bug this replaced: a --scope elsewhere in the file
    # (e.g. a sibling job) must not satisfy the check for the e2e-verify job specifically.
    text = "  other-job:\n    run: something --scope foo --base bar\n\n" + UNWIRED
    assert find_missing_wiring(text) is not None


def test_rejects_a_base_flag_outside_the_e2e_verify_block():
    # A --base in a sibling job (e.g. coverage-changed) must not satisfy the e2e-verify check:
    # the block is extracted first, so only the e2e-verify job's own wiring counts (#319).
    text = "  other-job:\n    run: something --base origin/main\n\n" + MISSING_BASE_FLAG_ONLY
    assert find_missing_wiring(text) == _BASE_ERROR


def test_command_echoes_on_a_fully_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "appends detect's extra-scope/exclude roots" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(UNWIRED)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "#294" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
