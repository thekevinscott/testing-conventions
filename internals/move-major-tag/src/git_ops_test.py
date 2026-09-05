"""Unit tests for the git boundary.

git is the one external, mocked as a `subprocess.run` fake that dispatches on the git subcommand
and records every argv and keyword, so the assertions pin the git commands actually issued.
"""
from unittest.mock import patch

import pytest

import git_ops


class _Result:
    """A `subprocess.CompletedProcess` stand-in carrying the one field callers read."""

    def __init__(self, returncode):
        self.returncode = returncode

    def check_returncode(self):
        if self.returncode:
            raise _GitFailed(self.returncode)


class _GitFailed(Exception):
    """What the fake raises where `subprocess.CalledProcessError` would."""


class _Git:
    """Records each `subprocess.run` call and answers by git subcommand."""

    def __init__(self, **returncodes):
        self.argv = []
        self.kwargs = []
        self._returncodes = returncodes

    def __call__(self, argv, **kwargs):
        self.argv.append(argv)
        self.kwargs.append(kwargs)
        return _Result(self._returncodes.get(argv[1], 0))


@pytest.fixture
def git():
    """Patch the one external — `subprocess.run` — and yield the recording fake."""
    fake = _Git()
    with patch("git_ops.subprocess.run", fake):
        yield fake


def test_git_runs_the_arguments_under_git(git):
    git_ops._git("status", "--short")
    assert git.argv == [["git", "status", "--short"]]


def test_git_asks_subprocess_to_capture_and_decode(git):
    git_ops._git("status")
    assert git.kwargs == [{"capture_output": True, "text": True}]


def test_fetch_tags_force_fetches_tags_from_origin(git):
    git_ops.fetch_tags()
    assert git.argv == [["git", "fetch", "--force", "--tags", "origin"]]


def test_fetch_tags_raises_when_the_fetch_fails(git):
    git._returncodes["fetch"] = 1
    with pytest.raises(_GitFailed):
        git_ops.fetch_tags()


def test_tag_exists_verifies_the_tag_ref(git):
    git_ops.tag_exists("v0")
    assert git.argv == [["git", "rev-parse", "-q", "--verify", "refs/tags/v0"]]


def test_tag_exists_is_true_when_the_ref_resolves(git):
    assert git_ops.tag_exists("v0") is True


def test_tag_exists_is_false_when_the_ref_does_not_resolve(git):
    git._returncodes["rev-parse"] = 1
    assert git_ops.tag_exists("v0") is False


def test_tag_exists_is_false_when_git_dies_on_a_signal(git):
    git._returncodes["rev-parse"] = -9
    assert git_ops.tag_exists("v0") is False


def test_is_ancestor_asks_merge_base_with_the_ancestor_first(git):
    git_ops.is_ancestor("oldsha", "v0")
    assert git.argv == [["git", "merge-base", "--is-ancestor", "oldsha", "v0"]]


def test_is_ancestor_is_true_when_merge_base_succeeds(git):
    assert git_ops.is_ancestor("oldsha", "v0") is True


def test_is_ancestor_is_false_when_merge_base_fails(git):
    git._returncodes["merge-base"] = 1
    assert git_ops.is_ancestor("newsha", "v0") is False


def test_is_ancestor_is_false_when_git_dies_on_a_signal(git):
    git._returncodes["merge-base"] = -9
    assert git_ops.is_ancestor("newsha", "v0") is False


def test_move_tag_force_moves_the_tag_locally(git):
    git_ops.move_tag("v0", "deadbeef")
    assert git.argv == [["git", "tag", "-f", "v0", "deadbeef"]]


def test_move_tag_raises_when_the_tag_write_fails(git):
    git._returncodes["tag"] = 1
    with pytest.raises(_GitFailed):
        git_ops.move_tag("v0", "deadbeef")


def test_push_tag_force_pushes_the_tag_ref_to_origin(git):
    git_ops.push_tag("v0")
    assert git.argv == [["git", "push", "-f", "origin", "refs/tags/v0"]]


def test_push_tag_raises_when_the_push_fails(git):
    git._returncodes["push"] = 1
    with pytest.raises(_GitFailed):
        git_ops.push_tag("v0")
