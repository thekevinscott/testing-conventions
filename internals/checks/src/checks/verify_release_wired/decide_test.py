"""Colocated unit tests for the verify-release-wired decision (isolation — pure, no I/O).

The decision is driven directly across each branch (missing file, no layout check, no suite
dispatch, a suite workflow un-dispatched, no move job, the move job ungated, clean), so every
early-return and the clean path are pinned. Only the unit under test is imported.
"""
from checks.verify_release_wired.decide import check_move_gated_on_verification

# A fully-wired move-major-tag.yml: a verified move job that needs the two verification jobs, both
# verification steps present, and both suite workflows dispatched.
WIRED = """\
jobs:
  context:
    steps:
      - run: uv run --project internals/checks tc-checks verify-release resolve-version "$SHA"
  verify-layout:
    steps:
      - run: uv run --project internals/checks tc-checks verify-release check-layout "$SHA"
  verify-suite:
    steps:
      - run: uv run --project internals/checks tc-checks verify-release dispatch-and-wait "$SHA" "$V" testing-conventions-selftest.yml dogfood.yml
  move-v0:
    needs: [context, verify-layout, verify-suite]
    steps:
      - run: python3 internals/move-major-tag/src/move_major_tag.py
"""


def test_error_when_the_file_is_absent():
    assert "no workflow advances @v0" in check_move_gated_on_verification(None)


def test_error_when_the_layout_check_is_missing():
    text = WIRED.replace('tc-checks verify-release check-layout "$SHA"', "echo skip")
    assert "layout check" in check_move_gated_on_verification(text)


def test_error_when_the_suite_dispatch_is_missing():
    text = WIRED.replace("tc-checks verify-release dispatch-and-wait", "echo skip")
    assert "suite dispatch" in check_move_gated_on_verification(text)


def test_error_when_a_suite_workflow_is_not_dispatched():
    # Replace with a name that does not *contain* "dogfood.yml" as a substring.
    text = WIRED.replace("dogfood.yml", "other.yml")
    assert "dogfood.yml" in check_move_gated_on_verification(text)


def test_error_when_no_job_runs_the_move_helper():
    text = WIRED.replace("python3 internals/move-major-tag/src/move_major_tag.py", "echo done")
    assert "move_major_tag.py" in check_move_gated_on_verification(text)


def test_error_when_the_move_job_is_not_gated_on_the_verify_jobs():
    # The false-negative a file-wide substring check would miss: the verify jobs exist and run,
    # but the move job doesn't `needs:` them, so a red verification wouldn't stop the move.
    text = WIRED.replace("    needs: [context, verify-layout, verify-suite]\n", "")
    error = check_move_gated_on_verification(text)
    assert "move-v0" in error
    assert "verify-layout" in error
    assert "verify-suite" in error


def test_error_when_the_move_job_needs_only_one_verify_job():
    text = WIRED.replace(
        "    needs: [context, verify-layout, verify-suite]\n",
        "    needs: [context, verify-layout]\n",
    )
    error = check_move_gated_on_verification(text)
    assert "verify-suite" in error
    assert "verify-layout" not in error


def test_clean_when_fully_wired_and_gated():
    assert check_move_gated_on_verification(WIRED) is None
