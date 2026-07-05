"""Colocated unit tests for check_rolling_release_wired.

Unit-level: the two pure inspections over crafted constants (no I/O). Each failure mode of the
move-major-tag validation is exercised independently, plus the release inline-move check both
ways. `main` and the `__main__` guard are covered by the e2e suite under `tests/e2e/`.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import check_rolling_release_wired as m  # noqa: E402

GATED = "on:\n  workflow_run:\n    workflows: [Release]\nconcurrency:\n  group: move-major-tag\n"


def test_move_tag_absent_is_reported():
    msg = m.check_move_major_tag(None)
    assert msg is not None
    assert "no dedicated advance workflow" in msg


def test_move_tag_without_workflow_run_is_reported():
    msg = m.check_move_major_tag("on:\n  push:\nconcurrency:\n  group: x\n")
    assert msg is not None
    assert "workflow_run" in msg


def test_move_tag_without_concurrency_is_reported():
    msg = m.check_move_major_tag("on:\n  workflow_run:\n    workflows: [Release]\n")
    assert msg is not None
    assert "concurrency" in msg


def test_move_tag_gated_and_guarded_is_ok():
    assert m.check_move_major_tag(GATED) is None


def test_release_moving_tag_inline_is_reported():
    msg = m.check_release_no_inline_tag_move("run: git tag -f v0 $SHA\n")
    assert msg is not None
    assert "inline" in msg


def test_release_without_inline_tag_move_is_ok():
    assert m.check_release_no_inline_tag_move("run: npm publish\n") is None


def test_path_from_argv_uses_default_when_index_absent():
    assert m.path_from_argv(["prog"], 1, "the-default") == "the-default"


def test_path_from_argv_prefers_the_explicit_argument():
    assert m.path_from_argv(["prog", "a", "b"], 2, "the-default") == "b"
