"""Colocated unit tests for `@path` resolution (isolation — the read is injected).

The read is a dict lookup, so the walk is exercised without a filesystem. Each case asserts on the
resolved content, which is what the budget is measured against.
"""
from checks.agents_md_size.resolve_agents_md.resolve_agents_md import resolve_agents_md


def _resolve(files, entry="AGENTS.md", **kwargs):
    return resolve_agents_md(entry, files.get, **kwargs)


def test_a_document_with_no_imports_resolves_to_itself():
    assert _resolve({"AGENTS.md": "solo\n"}) == "solo\n"


def test_a_missing_entry_resolves_to_nothing():
    assert _resolve({}) == ""


def test_an_empty_entry_resolves_to_nothing_without_being_treated_as_missing():
    assert _resolve({"AGENTS.md": ""}) == ""


def test_an_imported_file_is_appended_to_the_importer():
    assert _resolve({"AGENTS.md": "@a.md\n", "a.md": "aye\n"}) == "@a.md\naye\n"


def test_imports_are_appended_in_document_order():
    files = {"AGENTS.md": "@a.md\n@b.md\n", "a.md": "aye\n", "b.md": "bee\n"}
    assert _resolve(files) == "@a.md\n@b.md\naye\nbee\n"


def test_a_missing_import_is_skipped_rather_than_failing():
    assert _resolve({"AGENTS.md": "@gone.md\n"}) == "@gone.md\n"


def test_a_file_imported_twice_is_counted_once():
    assert _resolve({"AGENTS.md": "@a.md\n@a.md\n", "a.md": "aye\n"}) == "@a.md\n@a.md\naye\n"


def test_a_repeated_import_does_not_stop_the_imports_after_it():
    # Same importing document, so skipping the repeat has to resume the walk rather than leave it.
    files = {"AGENTS.md": "@a.md\n@a.md\n@b.md\n", "a.md": "aye\n", "b.md": "bee\n"}
    assert _resolve(files) == "@a.md\n@a.md\n@b.md\naye\nbee\n"


def test_a_missing_import_does_not_stop_the_imports_after_it():
    files = {"AGENTS.md": "@gone.md\n@b.md\n", "b.md": "bee\n"}
    assert _resolve(files) == "@gone.md\n@b.md\nbee\n"


def test_a_cycle_terminates():
    assert _resolve({"AGENTS.md": "@a.md\n", "a.md": "@AGENTS.md\n"}) == "@a.md\n@AGENTS.md\n"


def test_an_import_of_an_import_is_followed():
    files = {"AGENTS.md": "@a.md\n", "a.md": "@b.md\n", "b.md": "bee\n"}
    assert _resolve(files) == "@a.md\n@b.md\nbee\n"


# A literal chain, written out rather than built from MAX_DEPTH: a test that generates its
# fixture from the constant moves with the constant and pins no particular depth.
_CHAIN = {
    "AGENTS.md": "@1.md\n",
    "1.md": "@2.md\n",
    "2.md": "@3.md\n",
    "3.md": "@4.md\n",
    "4.md": "@5.md\n",
    "5.md": "@6.md\n",
    "6.md": "seventh file\n",
}


def test_the_fifth_import_in_a_chain_is_still_loaded():
    assert "@6.md\n" in _resolve(_CHAIN)


def test_the_sixth_import_in_a_chain_is_not_loaded():
    assert "seventh file\n" not in _resolve(_CHAIN)


def test_the_depth_is_an_argument_the_caller_can_lower():
    files = {"AGENTS.md": "@a.md\n", "a.md": "@b.md\n", "b.md": "bee\n"}
    assert _resolve(files, max_depth=1) == "@a.md\n@b.md\n"


def test_a_target_is_resolved_against_the_importing_file():
    files = {"pkg/AGENTS.md": "@style.md\n", "pkg/style.md": "styled\n"}
    assert _resolve(files, entry="pkg/AGENTS.md") == "@style.md\nstyled\n"


def test_a_target_outside_the_checkout_is_never_read():
    asked = []

    def read(path):
        asked.append(path)
        return {"AGENTS.md": "@~/outside.md\n"}.get(path)

    resolve_agents_md("AGENTS.md", read)
    assert asked == ["AGENTS.md"]
