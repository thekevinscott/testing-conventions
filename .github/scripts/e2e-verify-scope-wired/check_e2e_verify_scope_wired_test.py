"""Colocated unit tests for check_e2e_verify_scope_wired.

Runs `main` directly (argv both with and without an explicit path) and the real `__main__`
guard via `runpy`, in-process — no subprocess — so the entry point stays on the measured
path, matching the existing `detect_e2e_test.py` convention in this repo.
"""
import os
import runpy
import sys
from pathlib import Path
from unittest.mock import patch

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_e2e_verify_scope_wired as m  # noqa: E402

SCRIPT = Path(__file__).resolve().parent / "check_e2e_verify_scope_wired.py"

WIRED = """\
  e2e-verify:
    name: E2E attestation freshness (e2e verify)
    if: ${{ github.event_name == 'pull_request' && (inputs.gates == '' || contains(inputs.gates, '"e2e-verify"')) }}
    steps:
      - uses: actions/checkout@v6
      - name: Verify the e2e attestation is current
        env:
          PACKAGE_ROOT: ${{ needs.detect.outputs.package_root }}
          SCAN_PATH: ${{ inputs.path }}
          BASE: ${{ inputs.base }}
        run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE"

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
      SCAN_PATH: ${{ inputs.path }}
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
      SCAN_PATH: ${{ inputs.path }}
      BASE: ${{ inputs.base }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH"

  packaging:
    name: Packaging
"""

MISSING_BASE_ENV_ONLY = """\
  e2e-verify:
    if: ${{ github.event_name == 'pull_request' }}
    env:
      SCAN_PATH: ${{ inputs.path }}
      BASE: something-else
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE"

  packaging:
    name: Packaging
"""

MISSING_PULL_REQUEST_GATE = """\
  e2e-verify:
    if: ${{ inputs.gates == '' || contains(inputs.gates, '"e2e-verify"') }}
    env:
      SCAN_PATH: ${{ inputs.path }}
      BASE: ${{ inputs.base }}
    run: npx -y testing-conventions e2e verify "$PACKAGE_ROOT" --scope "$SCAN_PATH" --base "$BASE"

  packaging:
    name: Packaging
"""


def test_finds_no_missing_wiring_when_scope_base_and_gate_are_all_present():
    assert m.find_missing_wiring(WIRED) is None


def test_finds_missing_wiring_when_scope_is_absent():
    assert m.find_missing_wiring(UNWIRED) is not None


def test_finds_missing_wiring_when_only_the_scope_flag_is_absent():
    assert m.find_missing_wiring(MISSING_SCOPE_FLAG_ONLY) == m._SCOPE_ERROR


def test_finds_missing_wiring_when_only_scan_path_is_unbound():
    assert m.find_missing_wiring(MISSING_SCAN_PATH_ONLY) == m._SCOPE_ERROR


def test_finds_missing_wiring_when_only_the_base_flag_is_absent():
    assert m.find_missing_wiring(MISSING_BASE_FLAG_ONLY) == m._BASE_ERROR


def test_finds_missing_wiring_when_only_the_base_env_is_unbound():
    assert m.find_missing_wiring(MISSING_BASE_ENV_ONLY) == m._BASE_ERROR


def test_finds_missing_wiring_when_the_pull_request_gate_is_absent():
    assert m.find_missing_wiring(MISSING_PULL_REQUEST_GATE) == m._GATE_ERROR


def test_extract_e2e_verify_block_stops_before_the_next_job():
    block = m.extract_e2e_verify_block(WIRED)
    assert "e2e-verify:" in block
    assert "packaging:" not in block


def test_extract_e2e_verify_block_is_empty_when_the_job_is_absent():
    assert m.extract_e2e_verify_block("  packaging:\n    name: Packaging\n") == ""


def test_rejects_a_scope_flag_outside_the_e2e_verify_block():
    # Regression for the actual #294 wiring bug this replaced: a --scope elsewhere in the file
    # (e.g. a sibling job) must not satisfy the check for the e2e-verify job specifically.
    text = "  other-job:\n    run: something --scope foo --base bar\n\n" + UNWIRED
    assert m.find_missing_wiring(text) is not None


def test_rejects_a_base_flag_outside_the_e2e_verify_block():
    # A --base in a sibling job (e.g. coverage-changed) must not satisfy the e2e-verify check:
    # the block is extracted first, so only the e2e-verify job's own wiring counts (#319).
    text = "  other-job:\n    run: something --base origin/main\n\n" + MISSING_BASE_FLAG_ONLY
    assert m.find_missing_wiring(text) == m._BASE_ERROR


@pytest.fixture
def chdir_to():
    """Temporarily change the working directory, restoring it afterward."""
    origin_cwd = os.getcwd()

    def go(path):
        os.chdir(path)

    try:
        yield go
    finally:
        os.chdir(origin_cwd)


@pytest.fixture
def argv_is():
    """Temporarily replace sys.argv, restoring it afterward."""
    with patch.object(sys, "argv", ["check_e2e_verify_scope_wired.py"]):
        yield


def test_main_with_explicit_path_returns_zero_and_prints_success_when_wired(tmp_path, capsys):
    workflow = tmp_path / "wired.yml"
    workflow.write_text(WIRED)
    assert m.main(["prog", str(workflow)]) == 0
    assert "diffs inputs.base on PRs" in capsys.readouterr().out


def test_main_with_explicit_path_returns_one_and_prints_error_when_unwired(tmp_path, capsys):
    workflow = tmp_path / "unwired.yml"
    workflow.write_text(UNWIRED)
    assert m.main(["prog", str(workflow)]) == 1
    assert "::error::" in capsys.readouterr().out


def test_main_with_no_argv_reads_the_default_workflow_path(tmp_path, chdir_to, capsys):
    default = tmp_path / m.DEFAULT_WORKFLOW
    default.parent.mkdir(parents=True)
    default.write_text(WIRED)
    chdir_to(tmp_path)
    assert m.main(["prog"]) == 0
    assert "diffs inputs.base on PRs" in capsys.readouterr().out


def test_main_with_an_empty_argv_also_reads_the_default_workflow_path(tmp_path, chdir_to, capsys):
    # Distinguishes len(argv) > 1 from len(argv) != 1: an empty argv has length 0, which is
    # neither > 1 nor == 1 — a `!= 1` mutant would try argv[1] on an empty list and crash.
    default = tmp_path / m.DEFAULT_WORKFLOW
    default.parent.mkdir(parents=True)
    default.write_text(WIRED)
    chdir_to(tmp_path)
    assert m.main([]) == 0
    assert "diffs inputs.base on PRs" in capsys.readouterr().out


def test_dunder_main_guard_runs_and_exits_nonzero_on_a_stale_workflow(tmp_path, chdir_to, argv_is, capsys):
    default = tmp_path / m.DEFAULT_WORKFLOW
    default.parent.mkdir(parents=True)
    default.write_text(UNWIRED)
    chdir_to(tmp_path)
    with pytest.raises(SystemExit) as exc_info:
        runpy.run_path(str(SCRIPT), run_name="__main__")
    assert exc_info.value.code == 1
    assert "::error::" in capsys.readouterr().out


def test_dunder_main_guard_does_not_run_for_a_lesser_module_name(tmp_path, chdir_to, argv_is, capsys):
    # Distinguishes == from <=: "__init__" < "__main__" lexically, so a `<=` mutant would still
    # treat this as "__main__" and wrongly invoke main() (raising SystemExit); == correctly
    # leaves the module a no-op import with no output and no exit.
    default = tmp_path / m.DEFAULT_WORKFLOW
    default.parent.mkdir(parents=True)
    default.write_text(UNWIRED)
    chdir_to(tmp_path)
    runpy.run_path(str(SCRIPT), run_name="__init__")
    assert capsys.readouterr().out == ""


def test_dunder_main_guard_runs_for_an_equal_but_distinct_module_name_object(
    tmp_path, chdir_to, argv_is, capsys
):
    # Distinguishes == from is: build "__main__" at runtime (not the module's compiled literal),
    # so it's equal in value but not necessarily the same object. An `is` mutant would treat the
    # names as different and skip main(); == correctly matches on value and runs it.
    default = tmp_path / m.DEFAULT_WORKFLOW
    default.parent.mkdir(parents=True)
    default.write_text(UNWIRED)
    chdir_to(tmp_path)
    run_name = "".join(["__", "main", "__"])
    with pytest.raises(SystemExit) as exc_info:
        runpy.run_path(str(SCRIPT), run_name=run_name)
    assert exc_info.value.code == 1
    assert "::error::" in capsys.readouterr().out
