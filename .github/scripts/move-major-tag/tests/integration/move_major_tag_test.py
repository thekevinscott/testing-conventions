"""Integration tests for `move_major_tag.advance` / `decide`.

Per the standard, an integration test runs first-party code for real and mocks the externals.
The one external here is git, so each test patches the boundary functions
(`fetch_tags` / `tag_exists` / `is_ancestor` / `move_tag` / `push_tag`) and drives the real
forward-only orchestration, asserting the action returned and which side effects ran.
"""
import sys
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[2]))  # .github/scripts/move-major-tag
import move_major_tag as m  # noqa: E402


def _advance(tag, sha, *, exists, ancestor, push=True):
    """Run advance() with the git boundary fully mocked to the given facts.

    Returns (action, mocks) where mocks names each patched boundary fn for call assertions.
    """
    with patch.object(m, "fetch_tags") as fetch, \
            patch.object(m, "tag_exists", return_value=exists), \
            patch.object(m, "is_ancestor", return_value=ancestor) as is_ancestor, \
            patch.object(m, "move_tag") as move_tag, \
            patch.object(m, "push_tag") as push_tag:
        action = m.advance(tag, sha, push=push)
    return action, {
        "fetch": fetch,
        "is_ancestor": is_ancestor,
        "move_tag": move_tag,
        "push_tag": push_tag,
    }


# --- the pure forward-only decision ---

def test_decide_bootstrap_when_tag_absent():
    assert m.decide(exists=False, sha_behind_or_at_tag=False) == "bootstrap"


def test_decide_noop_when_sha_at_or_behind_tag():
    assert m.decide(exists=True, sha_behind_or_at_tag=True) == "noop"


def test_decide_advance_when_sha_ahead_of_tag():
    assert m.decide(exists=True, sha_behind_or_at_tag=False) == "advance"


# --- advance() orchestration (git boundary mocked) ---

def test_advance_bootstraps_when_tag_absent():
    # No tag yet: create + push it, and never run the ancestry check (nothing to compare to).
    action, mocks = _advance("v0", "deadbeef", exists=False, ancestor=False)
    assert action == "bootstrap"
    mocks["fetch"].assert_called_once()
    mocks["is_ancestor"].assert_not_called()
    mocks["move_tag"].assert_called_once_with("v0", "deadbeef")
    mocks["push_tag"].assert_called_once_with("v0")


def test_advance_moves_forward_when_sha_ahead():
    action, mocks = _advance("v0", "newsha", exists=True, ancestor=False)
    assert action == "advance"
    mocks["move_tag"].assert_called_once_with("v0", "newsha")
    mocks["push_tag"].assert_called_once_with("v0")


def test_advance_is_noop_when_sha_at_or_behind_tag():
    # Out-of-order / re-run release: the released sha is at or behind @v0 — never rewind.
    action, mocks = _advance("v0", "oldsha", exists=True, ancestor=True)
    assert action == "noop"
    mocks["move_tag"].assert_not_called()
    mocks["push_tag"].assert_not_called()


def test_advance_can_skip_the_push():
    # push=False moves the tag locally but does not push — the seam the e2e test relies on.
    action, mocks = _advance("v0", "newsha", exists=True, ancestor=False, push=False)
    assert action == "advance"
    mocks["move_tag"].assert_called_once_with("v0", "newsha")
    mocks["push_tag"].assert_not_called()
