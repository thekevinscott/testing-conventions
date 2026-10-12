"""Colocated unit tests for the transitive reference walk (isolation — injected extractor)."""
from pathlib import Path

from checks.utils.scope.closure import closure


def _refs_from(table):
    """A fake extractor reading a `{path: [ref, ...]}` table, recording every path it was asked for."""
    asked = []

    def extract_refs(path):
        asked.append(path)
        return [Path(ref) for ref in table.get(str(path), [])]

    extract_refs.asked = asked
    return extract_refs


def test_a_seed_with_no_references_is_itself_alone():
    assert closure(Path("a.md"), _refs_from({})) == [Path("a.md")]


def test_a_seed_written_as_a_string_is_a_path():
    assert closure("a.md", _refs_from({})) == [Path("a.md")]


def test_a_reference_follows_the_seed():
    walked = closure(Path("a.md"), _refs_from({"a.md": ["b.md"]}))
    assert walked == [Path("a.md"), Path("b.md")]


def test_a_reference_is_followed_transitively():
    extract = _refs_from({"a.md": ["b.md"], "b.md": ["c.md"]})
    assert closure(Path("a.md"), extract) == [Path("a.md"), Path("b.md"), Path("c.md")]


def test_the_walk_is_breadth_first():
    extract = _refs_from({"a.md": ["b.md", "c.md"], "b.md": ["d.md"]})
    assert closure(Path("a.md"), extract) == [
        Path("a.md"),
        Path("b.md"),
        Path("c.md"),
        Path("d.md"),
    ]


def test_a_reference_written_as_a_string_is_a_path():
    def extract_refs(path):
        return ["b.md"] if str(path) == "a.md" else []

    assert closure(Path("a.md"), extract_refs) == [Path("a.md"), Path("b.md")]


def test_a_cycle_terminates():
    extract = _refs_from({"a.md": ["b.md"], "b.md": ["a.md"]})
    assert closure(Path("a.md"), extract) == [Path("a.md"), Path("b.md")]


def test_a_self_reference_yields_the_seed_once():
    assert closure(Path("a.md"), _refs_from({"a.md": ["a.md"]})) == [Path("a.md")]


def test_a_reference_two_paths_share_is_yielded_once():
    extract = _refs_from(
        {"a.md": ["b.md", "c.md"], "b.md": ["shared.md"], "c.md": ["shared.md"]}
    )
    assert closure(Path("a.md"), extract) == [
        Path("a.md"),
        Path("b.md"),
        Path("c.md"),
        Path("shared.md"),
    ]


def test_a_reference_two_paths_share_is_asked_for_once():
    extract = _refs_from(
        {"a.md": ["b.md", "c.md"], "b.md": ["shared.md"], "c.md": ["shared.md"]}
    )
    closure(Path("a.md"), extract)
    assert extract.asked.count(Path("shared.md")) == 1


def test_a_repeated_reference_does_not_stop_the_ones_after_it():
    extract = _refs_from({"a.md": ["a.md", "b.md"]})
    assert closure(Path("a.md"), extract) == [Path("a.md"), Path("b.md")]
