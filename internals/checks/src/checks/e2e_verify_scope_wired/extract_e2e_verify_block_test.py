"""Colocated unit tests for the e2e-verify block extraction (isolation — pure, no I/O)."""
from checks.e2e_verify_scope_wired.extract_e2e_verify_block import extract_e2e_verify_block

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


def test_extract_e2e_verify_block_stops_before_the_next_job():
    block = extract_e2e_verify_block(WIRED)
    assert "e2e-verify:" in block
    assert "packaging:" not in block


def test_extract_e2e_verify_block_is_empty_when_the_job_is_absent():
    assert extract_e2e_verify_block("  packaging:\n    name: Packaging\n") == ""


def test_extract_e2e_verify_block_runs_to_end_when_no_next_job_follows():
    # No following `  packaging:` header, so the block extends to the end of the file.
    text = '  e2e-verify:\n    run: something --scope x --base y $EXTRA_SCOPE $EXCLUDE\n'
    assert extract_e2e_verify_block(text) == text
