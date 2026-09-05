"""Unit tests for the tag-advance orchestration.

The git boundary and the decision are collaborators, patched on this module in a fixture, so the
assertions pin what `advance` asks of each one and what it does with the answer.
"""
from types import SimpleNamespace
from unittest.mock import patch

import pytest

import advance as unit


@pytest.fixture
def collaborators():
    """Patch the git boundary and the decision, yielding the mocks for setup + assertions."""
    with patch.object(unit, "fetch_tags") as fetch_tags, \
            patch.object(unit, "tag_exists") as tag_exists, \
            patch.object(unit, "is_ancestor") as is_ancestor, \
            patch.object(unit, "move_tag") as move_tag, \
            patch.object(unit, "push_tag") as push_tag, \
            patch.object(unit, "decide") as decide:
        decide.return_value = "noop"
        yield SimpleNamespace(
            fetch_tags=fetch_tags,
            tag_exists=tag_exists,
            is_ancestor=is_ancestor,
            move_tag=move_tag,
            push_tag=push_tag,
            decide=decide,
        )


def test_advance_syncs_the_tags_before_reading_them(collaborators):
    unit.advance("v0", "newsha")
    collaborators.fetch_tags.assert_called_once_with()


def test_advance_looks_the_tag_up_by_name(collaborators):
    unit.advance("v0", "newsha")
    collaborators.tag_exists.assert_called_once_with("v0")


def test_advance_compares_the_sha_against_the_tag(collaborators):
    collaborators.tag_exists.return_value = True
    unit.advance("v0", "newsha")
    collaborators.is_ancestor.assert_called_once_with("newsha", "v0")


def test_advance_skips_the_ancestry_check_when_the_tag_is_absent(collaborators):
    collaborators.tag_exists.return_value = False
    unit.advance("v0", "newsha")
    collaborators.is_ancestor.assert_not_called()


def test_advance_reports_the_absent_tag_as_not_behind(collaborators):
    collaborators.tag_exists.return_value = False
    unit.advance("v0", "newsha")
    collaborators.decide.assert_called_once_with(exists=False, sha_behind_or_at_tag=False)


def test_advance_passes_both_git_facts_to_the_decision(collaborators):
    collaborators.tag_exists.return_value = True
    collaborators.is_ancestor.return_value = True
    unit.advance("v0", "oldsha")
    collaborators.decide.assert_called_once_with(exists=True, sha_behind_or_at_tag=True)


def test_advance_returns_the_decided_action(collaborators):
    collaborators.decide.return_value = "advance"
    assert unit.advance("v0", "newsha") == "advance"


def test_advance_moves_the_tag_when_the_decision_advances(collaborators):
    collaborators.decide.return_value = "advance"
    unit.advance("v0", "newsha")
    collaborators.move_tag.assert_called_once_with("v0", "newsha")


def test_advance_moves_the_tag_when_the_decision_bootstraps(collaborators):
    collaborators.decide.return_value = "bootstrap"
    unit.advance("v0", "deadbeef")
    collaborators.move_tag.assert_called_once_with("v0", "deadbeef")


def test_advance_pushes_the_tag_it_moved(collaborators):
    collaborators.decide.return_value = "advance"
    unit.advance("v0", "newsha")
    collaborators.push_tag.assert_called_once_with("v0")


def test_advance_writes_nothing_on_a_noop(collaborators):
    collaborators.decide.return_value = "noop"
    unit.advance("v0", "oldsha")
    collaborators.move_tag.assert_not_called()
    collaborators.push_tag.assert_not_called()


def test_advance_can_skip_the_push(collaborators):
    collaborators.decide.return_value = "advance"
    unit.advance("v0", "newsha", push=False)
    collaborators.push_tag.assert_not_called()


def test_advance_moves_the_tag_locally_when_the_push_is_skipped(collaborators):
    collaborators.decide.return_value = "advance"
    unit.advance("v0", "newsha", push=False)
    collaborators.move_tag.assert_called_once_with("v0", "newsha")


def test_advance_takes_push_by_keyword_only(collaborators):
    with pytest.raises(TypeError):
        unit.advance("v0", "newsha", False)


def test_the_writing_actions_are_bootstrap_and_advance():
    assert unit.WRITING_ACTIONS == frozenset({"bootstrap", "advance"})
