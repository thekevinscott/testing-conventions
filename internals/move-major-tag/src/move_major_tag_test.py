"""Colocated unit tests for move_major_tag.

Unit-level: the pure forward-only decision, exercised in isolation (no git, no mocks). The
orchestration (`advance`) is covered by the integration suite with the git boundary mocked, and
the end-to-end path by the e2e suite against a real repo — both under `tests/`.
"""
import move_major_tag as m


def test_decide_bootstraps_when_tag_absent():
    assert m.decide(exists=False, sha_behind_or_at_tag=False) == "bootstrap"


def test_decide_is_a_noop_when_sha_at_or_behind_tag():
    assert m.decide(exists=True, sha_behind_or_at_tag=True) == "noop"


def test_decide_advances_when_sha_ahead_of_tag():
    assert m.decide(exists=True, sha_behind_or_at_tag=False) == "advance"
