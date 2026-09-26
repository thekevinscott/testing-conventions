"""Colocated unit tests for the import closure (isolation — the read is injected as a fake)."""
from checks.agents_md_size.closure import MAX_DEPTH, closure


def _read_from(files):
    """A fake read over a dict of path -> text; anything absent reads as missing.

    The walk is only ever meant to read a resolved repo-relative path, so an unresolved one
    raises rather than quietly reading as missing.
    """
    def read(path):
        if not isinstance(path, str):
            raise TypeError(f"the read was asked for an unresolved target: {path!r}")
        return files.get(path)

    return read


def _chain(depth):
    """An entry importing a file importing a file ..., `depth` links past the entry."""
    files = {"AGENTS.md": "@f1.md\n"}
    for step in range(1, depth):
        files[f"f{step}.md"] = f"@f{step + 1}.md\n"
    files[f"f{depth}.md"] = "leaf\n"
    return files


def test_a_file_with_no_imports_is_its_own_closure():
    assert closure("AGENTS.md", _read_from({"AGENTS.md": "prose\n"})) == [("AGENTS.md", "prose\n")]


def test_a_missing_entry_has_an_empty_closure():
    assert closure("AGENTS.md", _read_from({})) == []


def test_an_empty_entry_is_still_a_member():
    assert closure("AGENTS.md", _read_from({"AGENTS.md": ""})) == [("AGENTS.md", "")]


def test_an_imported_file_joins_the_closure_after_the_entry():
    files = {"AGENTS.md": "@a.md\n", "a.md": "aye\n"}
    assert closure("AGENTS.md", _read_from(files)) == [("AGENTS.md", "@a.md\n"), ("a.md", "aye\n")]


def test_an_import_is_followed_transitively():
    files = {"AGENTS.md": "@a.md\n", "a.md": "@b.md\n", "b.md": "bee\n"}
    assert [path for path, _ in closure("AGENTS.md", _read_from(files))] == [
        "AGENTS.md",
        "a.md",
        "b.md",
    ]


def test_an_import_resolves_against_its_own_containing_file():
    files = {"AGENTS.md": "@docs/a.md\n", "docs/a.md": "@b.md\n", "docs/b.md": "bee\n"}
    assert [path for path, _ in closure("AGENTS.md", _read_from(files))] == [
        "AGENTS.md",
        "docs/a.md",
        "docs/b.md",
    ]


def test_a_missing_import_is_skipped():
    files = {"AGENTS.md": "@gone.md\n@a.md\n", "a.md": "aye\n"}
    assert [path for path, _ in closure("AGENTS.md", _read_from(files))] == ["AGENTS.md", "a.md"]


def test_an_import_outside_the_checkout_is_skipped():
    files = {"AGENTS.md": "@~/global.md\n@a.md\n", "a.md": "aye\n"}
    assert [path for path, _ in closure("AGENTS.md", _read_from(files))] == ["AGENTS.md", "a.md"]


def test_a_cycle_terminates_with_each_file_counted_once():
    files = {"AGENTS.md": "@a.md\n", "a.md": "@AGENTS.md\n"}
    assert [path for path, _ in closure("AGENTS.md", _read_from(files))] == ["AGENTS.md", "a.md"]


def test_a_file_imported_twice_is_a_member_once():
    files = {"AGENTS.md": "@a.md\n@a.md\n@b.md\n", "a.md": "aye\n", "b.md": "bee\n"}
    assert [path for path, _ in closure("AGENTS.md", _read_from(files))] == [
        "AGENTS.md",
        "a.md",
        "b.md",
    ]


def test_a_chain_at_the_depth_cap_is_fully_loaded():
    files = _chain(MAX_DEPTH)
    assert len(closure("AGENTS.md", _read_from(files))) == MAX_DEPTH + 1


def test_a_chain_one_hop_past_the_cap_drops_its_last_file():
    files = _chain(MAX_DEPTH + 1)
    loaded = [path for path, _ in closure("AGENTS.md", _read_from(files))]
    assert len(loaded) == MAX_DEPTH + 1
    assert f"f{MAX_DEPTH + 1}.md" not in loaded


def test_the_depth_cap_is_the_five_hops_an_agent_follows():
    assert MAX_DEPTH == 5


def test_a_shallower_cap_stops_the_walk_sooner():
    files = _chain(MAX_DEPTH)
    assert [path for path, _ in closure("AGENTS.md", _read_from(files), max_depth=1)] == [
        "AGENTS.md",
        "f1.md",
    ]
