"""Colocated unit tests for the seeds-in-scope intersection (isolation — injected extractor and diff)."""
from pathlib import Path

from checks.utils.scope.seeds_in_scope import seeds_in_scope


def _refs_from(table):
    """A fake extractor reading a `{path: [ref, ...]}` table."""

    def extract_refs(path):
        return [Path(ref) for ref in table.get(str(path), [])]

    return extract_refs


def test_a_seed_whose_own_path_changed_is_in_scope():
    found = seeds_in_scope([Path("a.md")], _refs_from({}), [Path("a.md")])
    assert found == [Path("a.md")]


def test_a_seed_whose_reference_changed_is_in_scope():
    extract = _refs_from({"a.md": ["shared.md"]})
    assert seeds_in_scope([Path("a.md")], extract, [Path("shared.md")]) == [Path("a.md")]


def test_a_seed_whose_transitive_reference_changed_is_in_scope():
    extract = _refs_from({"a.md": ["b.md"], "b.md": ["deep.md"]})
    assert seeds_in_scope([Path("a.md")], extract, [Path("deep.md")]) == [Path("a.md")]


def test_a_seed_the_change_never_reaches_is_out_of_scope():
    extract = _refs_from({"a.md": ["b.md"]})
    assert seeds_in_scope([Path("a.md")], extract, [Path("elsewhere.md")]) == []


def test_only_the_seeds_the_change_reaches_come_back():
    extract = _refs_from({"a.md": ["shared.md"], "b.md": ["own.md"]})
    found = seeds_in_scope([Path("a.md"), Path("b.md")], extract, [Path("own.md")])
    assert found == [Path("b.md")]


def test_every_seed_the_change_reaches_comes_back_in_the_order_given():
    extract = _refs_from({"a.md": ["shared.md"], "b.md": ["shared.md"]})
    found = seeds_in_scope([Path("b.md"), Path("a.md")], extract, [Path("shared.md")])
    assert found == [Path("b.md"), Path("a.md")]


def test_no_seeds_are_no_scope():
    assert seeds_in_scope([], _refs_from({}), [Path("a.md")]) == []


def test_no_change_is_no_scope():
    assert seeds_in_scope([Path("a.md")], _refs_from({}), []) == []


def test_a_seed_and_a_changed_path_written_as_strings_are_paths():
    assert seeds_in_scope(["a.md"], _refs_from({}), ["a.md"]) == [Path("a.md")]


def test_a_changed_path_in_the_other_frame_is_an_error():
    try:
        seeds_in_scope([Path("/repo/a.md")], _refs_from({}), [Path("a.md")])
    except ValueError as error:
        assert "one frame" in str(error)
    else:
        raise AssertionError("a mixed frame must raise")
