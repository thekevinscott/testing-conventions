"""Colocated unit tests for the inline-tag-move decision (isolation — pure, no I/O)."""
from checks.rolling_release_wired.check_release_no_inline_tag_move import (
    check_release_no_inline_tag_move,
)


def test_release_error_when_it_moves_the_tag_inline():
    assert "inline" in check_release_no_inline_tag_move("run: git tag -f v0 $SHA\n")


def test_release_clean_when_it_moves_no_tag():
    assert check_release_no_inline_tag_move("run: npm publish\n") is None
