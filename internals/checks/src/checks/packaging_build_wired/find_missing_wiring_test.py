"""Colocated unit tests for the packaging wiring decision (isolation — pure, no I/O).

Each fixture drops exactly one wired fact, so every branch of the walk maps to one test; the
sibling fixture puts all the wiring in another job to prove only the packaging job's own lines
count.
"""
from checks.packaging_build_wired.find_missing_wiring import (
    _BUILD_ERROR,
    _CRATE_SCAN_ERROR,
    _GATE_ERROR,
    _PROVISION_ERROR,
    find_missing_wiring,
)

WIRED = """\
  packaging:
    name: Packaging (no test files in the built artifact)
    needs: detect
    if: ${{ (inputs.gates == '' || contains(inputs.gates, '"packaging"')) && (inputs.packaging_artifact != '' || needs.detect.outputs.packaging_build != '' || needs.detect.outputs.packaging_dist == 'true') }}
    steps:
      - if: ${{ inputs.packaging_artifact == '' && needs.detect.outputs.packaging_language == 'python' }}
        uses: astral-sh/setup-uv@v7
      - if: ${{ inputs.packaging_artifact == '' && needs.detect.outputs.packaging_build != '' }}
        name: Build the distribution (packaging_build)
        env:
          PACKAGING_BUILD: ${{ needs.detect.outputs.packaging_build }}
        run: |
          eval "$PACKAGING_BUILD"
      - name: Check the built distributions ship no test files
        run: |
          if [ -n "$PACKAGING_ARTIFACT" ]; then crate_root=_packaging_artifact; else crate_root="$pkg/target/package"; fi
          check rust "$crate_root"/**/*.crate
"""

# Each unwired variant keeps everything the earlier checks require, so the walk reaches the
# branch under test.
MISSING_GATE = WIRED.replace(
    "|| needs.detect.outputs.packaging_build != '' ", ""
)
MISSING_PROVISION = WIRED.replace(
    "needs.detect.outputs.packaging_language == 'python'", "true"
)
MISSING_BUILD_EVAL = WIRED.replace('eval "$PACKAGING_BUILD"', "true")
MISSING_BUILD_ENV = WIRED.replace(
    "PACKAGING_BUILD: ${{ needs.detect.outputs.packaging_build }}",
    "PACKAGING_BUILD: something-else",
)
MISSING_CRATE_SCAN = WIRED.replace("target/package", "dist")

# A `packaging_build` gate / provision / build living in a sibling job must not satisfy the
# check for the packaging job specifically — the block is extracted first.
SIBLING_ONLY = """\
  other-job:
    if: ${{ needs.detect.outputs.packaging_build != '' }}
    run: |
      eval "$PACKAGING_BUILD"  # PACKAGING_BUILD: needs.detect.outputs.packaging_build
      cat "$pkg/target/package"  # needs.detect.outputs.packaging_language

  packaging:
    name: Packaging
    steps:
      - run: echo nothing wired here
"""


def test_finds_no_missing_wiring_when_gate_provision_build_and_crate_scan_are_all_present():
    assert find_missing_wiring(WIRED) is None


def test_finds_missing_wiring_when_the_build_gate_is_absent():
    assert find_missing_wiring(MISSING_GATE) == _GATE_ERROR


def test_finds_missing_wiring_when_the_language_provision_is_absent():
    assert find_missing_wiring(MISSING_PROVISION) == _PROVISION_ERROR


def test_finds_missing_wiring_when_the_build_eval_is_absent():
    assert find_missing_wiring(MISSING_BUILD_EVAL) == _BUILD_ERROR


def test_finds_missing_wiring_when_the_build_env_is_unbound():
    assert find_missing_wiring(MISSING_BUILD_ENV) == _BUILD_ERROR


def test_finds_missing_wiring_when_the_crate_scan_is_absent():
    assert find_missing_wiring(MISSING_CRATE_SCAN) == _CRATE_SCAN_ERROR


def test_rejects_wiring_that_lives_outside_the_packaging_block():
    # The gate/provision/build all appear in a sibling job, but the packaging job itself wires
    # none of it — the block is extracted first, so the sibling's wiring must not count.
    assert find_missing_wiring(SIBLING_ONLY) == _GATE_ERROR
