"""Colocated unit tests for the changed-paths intersection."""
from checks.agents_md_size.touching import touching

ENTRY = ("AGENTS.md", [("AGENTS.md", "a"), ("agents/style.md", "b")])
OTHER = ("pkg/AGENTS.md", [("pkg/AGENTS.md", "c")])


def test_a_closure_whose_entry_changed_is_kept():
    assert touching([ENTRY], ["AGENTS.md"]) == [ENTRY]


def test_a_closure_whose_import_changed_is_kept():
    assert touching([ENTRY], ["agents/style.md"]) == [ENTRY]


def test_a_closure_no_changed_path_reaches_is_dropped():
    assert touching([ENTRY], ["README.md"]) == []


def test_nothing_changed_keeps_nothing():
    assert touching([ENTRY, OTHER], []) == []


def test_only_the_reached_closures_survive():
    assert touching([ENTRY, OTHER], ["pkg/AGENTS.md"]) == [OTHER]


def test_every_reached_closure_survives_in_order():
    assert touching([ENTRY, OTHER], ["AGENTS.md", "pkg/AGENTS.md"]) == [ENTRY, OTHER]


def test_an_empty_closure_list_stays_empty():
    assert touching([], ["AGENTS.md"]) == []
