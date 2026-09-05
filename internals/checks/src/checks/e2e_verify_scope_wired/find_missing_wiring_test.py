"""Colocated unit tests for the e2e-verify wiring decision (isolation — pure, no I/O).

Each fixture drops exactly one wired fact, so every branch of the walk maps to one test; two
block-scoping fixtures put the flag in a sibling job to prove only the e2e-verify job's own
lines count.
"""
from checks.e2e_verify_scope_wired.find_missing_wiring import (
    _BASE_ERROR,
    _EXCLUDE_ERROR,
    _EXTRA_SCOPE_ERROR,
    _GATE_ERROR,
    _SCOPE_ERROR,
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

# Each variant keeps everything the earlier checks require, so the walk reaches the new branch.
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


def test_rejects_a_scope_flag_outside_the_e2e_verify_block():
    # A --scope in a sibling job must not satisfy the check for the e2e-verify job.
    text = "  other-job:\n    run: something --scope foo --base bar\n\n" + UNWIRED
    assert find_missing_wiring(text) is not None


def test_rejects_a_base_flag_outside_the_e2e_verify_block():
    # The block is extracted first, so only the e2e-verify job's own `--base` counts.
    text = "  other-job:\n    run: something --base origin/main\n\n" + MISSING_BASE_FLAG_ONLY
    assert find_missing_wiring(text) == _BASE_ERROR
