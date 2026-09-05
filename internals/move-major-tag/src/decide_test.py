"""Unit tests for the forward-only decision: a pure function of two git facts."""
from decide import decide


def test_decide_bootstraps_when_tag_absent():
    assert decide(exists=False, sha_behind_or_at_tag=False) == "bootstrap"


def test_decide_bootstraps_even_when_the_ancestry_fact_says_behind():
    assert decide(exists=False, sha_behind_or_at_tag=True) == "bootstrap"


def test_decide_is_a_noop_when_sha_at_or_behind_tag():
    assert decide(exists=True, sha_behind_or_at_tag=True) == "noop"


def test_decide_advances_when_sha_ahead_of_tag():
    assert decide(exists=True, sha_behind_or_at_tag=False) == "advance"
