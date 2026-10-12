"""Colocated unit tests for the scope frame guard (isolation — pure)."""
from pathlib import Path

from checks.utils.scope.one_frame import one_frame


def test_absolute_paths_come_back_unchanged():
    paths = [Path("/repo/a.md"), Path("/repo/b.md")]
    assert one_frame(paths) == paths


def test_relative_paths_come_back_unchanged():
    paths = [Path("a.md"), Path("nested/b.md")]
    assert one_frame(paths) == paths


def test_one_path_comes_back_unchanged():
    assert one_frame([Path("a.md")]) == [Path("a.md")]


def test_no_paths_come_back_unchanged():
    assert one_frame([]) == []


def test_a_relative_path_among_absolute_ones_is_an_error():
    try:
        one_frame([Path("/repo/a.md"), Path("b.md")])
    except ValueError as error:
        assert "one frame" in str(error)
    else:
        raise AssertionError("a mixed frame must raise")


def test_an_absolute_path_among_relative_ones_is_an_error():
    try:
        one_frame([Path("a.md"), Path("/repo/b.md")])
    except ValueError as error:
        assert "absolute" in str(error)
        assert "repo-relative" in str(error)
    else:
        raise AssertionError("a mixed frame must raise")
