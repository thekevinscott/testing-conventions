"""Colocated unit tests for check_e2e_verify_scope_wired."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_e2e_verify_scope_wired as m  # noqa: E402

WIRED = """\
  e2e-verify:
    name: E2E attestation freshness (e2e verify)
    steps:
      - uses: actions/checkout@v6
      - name: Verify the e2e attestation is current
        env:
          PACKAGE_ROOT: ${{ needs.detect.outputs.package_root }}
          SCAN_PATH: ${{ inputs.path }}
        run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH"

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


def test_finds_no_missing_wiring_when_scope_is_passed():
    assert m.find_missing_wiring(WIRED) is None


def test_finds_missing_wiring_when_scope_is_absent():
    assert m.find_missing_wiring(UNWIRED) is not None


def test_extract_e2e_verify_block_stops_before_the_next_job():
    block = m.extract_e2e_verify_block(WIRED)
    assert "e2e-verify:" in block
    assert "packaging:" not in block


def test_extract_e2e_verify_block_is_empty_when_the_job_is_absent():
    assert m.extract_e2e_verify_block("  packaging:\n    name: Packaging\n") == ""


def test_rejects_a_scope_flag_outside_the_e2e_verify_block():
    # Regression for the actual #294 wiring bug this replaced: a --scope elsewhere in the file
    # (e.g. a sibling job) must not satisfy the check for the e2e-verify job specifically.
    text = "  other-job:\n    run: something --scope foo\n\n" + UNWIRED
    assert m.find_missing_wiring(text) is not None
