"""Colocated unit tests for the move-major-tag decision (isolation — pure, no I/O).

Driven directly across each branch (missing file, ungated, unguarded, clean), so every
early-return and the clean path are pinned.
"""
from checks.rolling_release_wired.decide import check_move_major_tag

GATED = "on:\n  workflow_run:\n    workflows: [Release]\nconcurrency:\n  group: move-major-tag\n"


def test_move_tag_error_when_the_file_is_absent():
    assert check_move_major_tag(None).endswith("has no dedicated advance workflow")


def test_move_tag_error_when_not_publish_gated():
    error = check_move_major_tag("concurrency:\n  group: x\n")
    assert "workflow_run" in error
    assert error.endswith("@v0 could advance before the binary publishes")


def test_move_tag_error_when_not_concurrency_guarded():
    error = check_move_major_tag("on:\n  workflow_run: {}\n")
    assert "concurrency group" in error
    assert error.endswith("out-of-order tag moves are unguarded")


def test_move_tag_clean_when_gated_and_guarded():
    assert check_move_major_tag(GATED) is None
