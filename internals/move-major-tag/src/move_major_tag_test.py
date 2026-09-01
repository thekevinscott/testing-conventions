"""Colocated unit tests for move_major_tag.

git is the one external, mocked as a `subprocess.run` fake that dispatches on the git subcommand
and records every argv and keyword. The boundary helpers, `advance`, and `main` all run for real
against it, so the assertions pin the git commands actually issued.
"""
import os
import runpy
from unittest.mock import patch

import pytest

import move_major_tag as m


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

    @property
    def subcommands(self):
        return [argv[1] for argv in self.argv]


@pytest.fixture
def git():
    """Patch the one external — `subprocess.run` — and yield the recording fake."""
    fake = _Git()
    with patch("move_major_tag.subprocess.run", fake):
        yield fake


def test_git_runs_the_arguments_under_git(git):
    m._git("status", "--short")
    assert git.argv == [["git", "status", "--short"]]


def test_git_asks_subprocess_to_capture_and_decode(git):
    m._git("status")
    assert git.kwargs == [{"capture_output": True, "text": True}]


def test_fetch_tags_force_fetches_tags_from_origin(git):
    m.fetch_tags()
    assert git.argv == [["git", "fetch", "--force", "--tags", "origin"]]


def test_fetch_tags_raises_when_the_fetch_fails(git):
    git._returncodes["fetch"] = 1
    with pytest.raises(_GitFailed):
        m.fetch_tags()


def test_tag_exists_verifies_the_tag_ref(git):
    m.tag_exists("v0")
    assert git.argv == [["git", "rev-parse", "-q", "--verify", "refs/tags/v0"]]


def test_tag_exists_is_true_when_the_ref_resolves(git):
    assert m.tag_exists("v0") is True


def test_tag_exists_is_false_when_the_ref_does_not_resolve(git):
    git._returncodes["rev-parse"] = 1
    assert m.tag_exists("v0") is False


def test_tag_exists_is_false_when_git_dies_on_a_signal(git):
    git._returncodes["rev-parse"] = -9
    assert m.tag_exists("v0") is False


def test_is_ancestor_asks_merge_base_with_the_ancestor_first(git):
    m.is_ancestor("oldsha", "v0")
    assert git.argv == [["git", "merge-base", "--is-ancestor", "oldsha", "v0"]]


def test_is_ancestor_is_true_when_merge_base_succeeds(git):
    assert m.is_ancestor("oldsha", "v0") is True


def test_is_ancestor_is_false_when_merge_base_fails(git):
    git._returncodes["merge-base"] = 1
    assert m.is_ancestor("newsha", "v0") is False


def test_is_ancestor_is_false_when_git_dies_on_a_signal(git):
    git._returncodes["merge-base"] = -9
    assert m.is_ancestor("newsha", "v0") is False


def test_move_tag_force_moves_the_tag_locally(git):
    m.move_tag("v0", "deadbeef")
    assert git.argv == [["git", "tag", "-f", "v0", "deadbeef"]]


def test_move_tag_raises_when_the_tag_write_fails(git):
    git._returncodes["tag"] = 1
    with pytest.raises(_GitFailed):
        m.move_tag("v0", "deadbeef")


def test_push_tag_force_pushes_the_tag_ref_to_origin(git):
    m.push_tag("v0")
    assert git.argv == [["git", "push", "-f", "origin", "refs/tags/v0"]]


def test_push_tag_raises_when_the_push_fails(git):
    git._returncodes["push"] = 1
    with pytest.raises(_GitFailed):
        m.push_tag("v0")


def test_decide_bootstraps_when_tag_absent():
    assert m.decide(exists=False, sha_behind_or_at_tag=False) == "bootstrap"


def test_decide_bootstraps_even_when_the_ancestry_fact_says_behind():
    assert m.decide(exists=False, sha_behind_or_at_tag=True) == "bootstrap"


def test_decide_is_a_noop_when_sha_at_or_behind_tag():
    assert m.decide(exists=True, sha_behind_or_at_tag=True) == "noop"


def test_decide_advances_when_sha_ahead_of_tag():
    assert m.decide(exists=True, sha_behind_or_at_tag=False) == "advance"


def test_advance_bootstraps_when_the_tag_is_absent(git):
    git._returncodes["rev-parse"] = 1
    assert m.advance("v0", "deadbeef") == "bootstrap"


def test_advance_skips_the_ancestry_check_when_the_tag_is_absent(git):
    git._returncodes["rev-parse"] = 1
    m.advance("v0", "deadbeef")
    assert git.subcommands == ["fetch", "rev-parse", "tag", "push"]


def test_advance_moves_and_pushes_the_tag_when_bootstrapping(git):
    git._returncodes["rev-parse"] = 1
    m.advance("v0", "deadbeef")
    assert git.argv[-2:] == [
        ["git", "tag", "-f", "v0", "deadbeef"],
        ["git", "push", "-f", "origin", "refs/tags/v0"],
    ]


def test_advance_moves_forward_when_the_sha_is_ahead_of_the_tag(git):
    git._returncodes["merge-base"] = 1
    assert m.advance("v0", "newsha") == "advance"


def test_advance_compares_the_sha_against_the_tag_before_moving(git):
    git._returncodes["merge-base"] = 1
    m.advance("v0", "newsha")
    assert git.argv[:3] == [
        ["git", "fetch", "--force", "--tags", "origin"],
        ["git", "rev-parse", "-q", "--verify", "refs/tags/v0"],
        ["git", "merge-base", "--is-ancestor", "newsha", "v0"],
    ]


def test_advance_is_a_noop_when_the_sha_is_at_or_behind_the_tag(git):
    assert m.advance("v0", "oldsha") == "noop"


def test_advance_writes_nothing_on_a_noop(git):
    m.advance("v0", "oldsha")
    assert git.subcommands == ["fetch", "rev-parse", "merge-base"]


def test_advance_can_skip_the_push(git):
    git._returncodes["merge-base"] = 1
    assert m.advance("v0", "newsha", push=False) == "advance"


def test_advance_moves_the_tag_locally_when_the_push_is_skipped(git):
    git._returncodes["merge-base"] = 1
    m.advance("v0", "newsha", push=False)
    assert git.subcommands == ["fetch", "rev-parse", "merge-base", "tag"]


def test_advance_takes_push_by_keyword_only(git):
    with pytest.raises(TypeError):
        m.advance("v0", "newsha", False)


def test_main_reports_a_missing_sha_as_an_error(capsys):
    with patch.dict(os.environ, {}, clear=True):
        assert m.main() == 1
    assert capsys.readouterr().out == (
        "::error::SHA is required (the released commit to advance the tag to)\n"
    )


def test_main_treats_a_whitespace_only_sha_as_missing():
    with patch.dict(os.environ, {"SHA": "   "}, clear=True):
        assert m.main() == 1


def test_main_advances_the_default_tag_when_none_is_named(git, capsys):
    git._returncodes["merge-base"] = 1
    with patch.dict(os.environ, {"SHA": "newsha"}, clear=True):
        assert m.main() == 0
    assert capsys.readouterr().out == "advanced v0 -> newsha\n"


def test_main_advances_the_named_tag(git):
    git._returncodes["merge-base"] = 1
    with patch.dict(os.environ, {"SHA": "newsha", "TAG": "v1"}, clear=True):
        m.main()
    assert git.argv[-1] == ["git", "push", "-f", "origin", "refs/tags/v1"]


def test_main_falls_back_to_the_default_tag_when_tag_is_whitespace(git):
    git._returncodes["merge-base"] = 1
    with patch.dict(os.environ, {"SHA": "newsha", "TAG": "  "}, clear=True):
        m.main()
    assert git.argv[-1] == ["git", "push", "-f", "origin", "refs/tags/v0"]


def test_main_strips_surrounding_whitespace_from_the_sha(git):
    git._returncodes["merge-base"] = 1
    with patch.dict(os.environ, {"SHA": "  newsha  "}, clear=True):
        m.main()
    assert git.argv[-2] == ["git", "tag", "-f", "v0", "newsha"]


def test_main_reports_a_bootstrap(git, capsys):
    git._returncodes["rev-parse"] = 1
    with patch.dict(os.environ, {"SHA": "deadbeef"}, clear=True):
        assert m.main() == 0
    assert capsys.readouterr().out == "v0 did not exist yet; bootstrapped it at deadbeef\n"


def test_main_reports_a_noop(git, capsys):
    with patch.dict(os.environ, {"SHA": "oldsha"}, clear=True):
        assert m.main() == 0
    assert capsys.readouterr().out == "v0 is already at or ahead of oldsha; nothing to do\n"


def test_running_the_module_as_a_script_exits_with_mains_status():
    # Built at runtime, so it is equal to "__main__" without being the same interned object:
    # comparing run names by identity would leave the guard dead for a caller that does the same.
    run_name = "".join(["__main", "__"])
    with patch.dict(os.environ, {}, clear=True):
        with pytest.raises(SystemExit) as exit_info:
            runpy.run_path(m.__file__, run_name=run_name)
    assert exit_info.value.code == 1


@pytest.mark.parametrize("run_name", ["__init__", "move_major_tag"])
def test_running_the_module_under_any_other_name_leaves_main_uncalled(run_name):
    with patch.dict(os.environ, {}, clear=True):
        assert runpy.run_path(m.__file__, run_name=run_name)["__name__"] == run_name
