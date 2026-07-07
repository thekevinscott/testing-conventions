"""Colocated unit tests for the packaging-build-wired check (isolation — no `CliRunner`).

The pure `find_missing_wiring` / `extract_packaging_block` decisions are driven directly over
crafted wired/unwired blocks; the `cli` command is driven through its `.callback` (the
undecorated function), so no `click.testing` collaborator is imported. Only the unit under test
is imported — a first-party collaborator (e.g. `CheckFailed`) would be flagged, so the raise
path is asserted against the propagated exception's `.message` instead.
"""
from checks.packaging_build_wired.cli import (
    REUSABLE_WORKFLOW,
    _BUILD_ERROR,
    _CRATE_SCAN_ERROR,
    _GATE_ERROR,
    _PROVISION_ERROR,
    cli,
    extract_packaging_block,
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


def test_declares_the_workflow_argument_defaulting_to_the_reusable_workflow():
    (argument,) = cli.params
    assert argument.name == "workflow"
    assert argument.default == REUSABLE_WORKFLOW


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


def test_extract_packaging_block_stops_before_the_next_job():
    text = WIRED + "\n  next-job:\n    name: After\n"
    block = extract_packaging_block(text)
    assert "packaging:" in block
    assert "next-job" not in block


def test_extract_packaging_block_runs_to_end_when_no_next_job_follows():
    assert extract_packaging_block(WIRED) == WIRED


def test_extract_packaging_block_is_empty_when_the_job_is_absent():
    assert extract_packaging_block("  other-job:\n    name: X\n") == ""


def test_rejects_wiring_that_lives_outside_the_packaging_block():
    # The gate/provision/build all appear in a sibling job, but the packaging job itself wires
    # none of it — the block is extracted first, so the sibling's wiring must not count.
    assert find_missing_wiring(SIBLING_ONLY) == _GATE_ERROR


def test_command_echoes_on_a_fully_wired_workflow(tmp_path, capsys):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(WIRED)
    cli.callback(workflow=str(workflow))
    assert "scans target/package" in capsys.readouterr().out


def test_command_raises_on_an_unwired_workflow(tmp_path):
    workflow = tmp_path / "wf.yml"
    workflow.write_text(MISSING_BUILD_EVAL)
    try:
        cli.callback(workflow=str(workflow))
    except Exception as error:  # noqa: BLE001 — CheckFailed is first-party; catch without importing it
        assert "#335" in error.message
    else:
        raise AssertionError("an unwired workflow must raise")
