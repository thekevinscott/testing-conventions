"""Integration tests for `advance`: the real orchestration, the git boundary mocked.

Per the standard, an integration test runs first-party code for real and mocks the externals.
git is the one external here, so the `git` fixture patches the boundary functions and yields the
mocks; each test configures their return values and asserts which side effects ran. The patching
lives in the fixture, never inline in a test body.
"""
from types import SimpleNamespace
from unittest.mock import patch

import pytest

import move_major_tag as m


@pytest.fixture
def git():
    """Patch the git boundary and yield the mocks for per-test setup + assertions."""
    with patch.object(m, "fetch_tags") as fetch, \
            patch.object(m, "tag_exists") as tag_exists, \
            patch.object(m, "is_ancestor") as is_ancestor, \
            patch.object(m, "move_tag") as move_tag, \
            patch.object(m, "push_tag") as push_tag:
        yield SimpleNamespace(
            fetch=fetch,
            tag_exists=tag_exists,
            is_ancestor=is_ancestor,
            move_tag=move_tag,
            push_tag=push_tag,
        )


def test_advance_bootstraps_when_tag_absent(git):
    # No tag yet: create + push it, and never run the ancestry check (nothing to compare to).
    git.tag_exists.return_value = False
    assert m.advance("v0", "deadbeef") == "bootstrap"
    git.fetch.assert_called_once()
    git.is_ancestor.assert_not_called()
    git.move_tag.assert_called_once_with("v0", "deadbeef")
    git.push_tag.assert_called_once_with("v0")


def test_advance_moves_forward_when_sha_ahead(git):
    git.tag_exists.return_value = True
    git.is_ancestor.return_value = False
    assert m.advance("v0", "newsha") == "advance"
    git.move_tag.assert_called_once_with("v0", "newsha")
    git.push_tag.assert_called_once_with("v0")


def test_advance_is_noop_when_sha_at_or_behind_tag(git):
    git.tag_exists.return_value = True
    git.is_ancestor.return_value = True
    assert m.advance("v0", "oldsha") == "noop"
    git.move_tag.assert_not_called()
    git.push_tag.assert_not_called()


def test_advance_can_skip_the_push(git):
    git.tag_exists.return_value = True
    git.is_ancestor.return_value = False
    assert m.advance("v0", "newsha", push=False) == "advance"
    git.move_tag.assert_called_once_with("v0", "newsha")
    git.push_tag.assert_not_called()
