"""Colocated unit tests for the `@path` import scan."""
from checks.agents_md_size.imports import import_targets


def test_a_bare_at_path_on_its_own_line_is_an_import():
    assert import_targets("@agents/style.md\n") == ["agents/style.md"]


def test_an_import_after_prose_on_the_same_line_is_found():
    assert import_targets("See @agents/style.md for details.\n") == ["agents/style.md"]


def test_every_import_is_returned_in_document_order():
    assert import_targets("@a.md\n@b.md\n") == ["a.md", "b.md"]


def test_a_repeated_import_is_returned_once_per_occurrence():
    assert import_targets("@a.md @a.md\n") == ["a.md", "a.md"]


def test_a_document_with_no_imports_yields_nothing():
    assert import_targets("# Title\n\nProse.\n") == []


def test_an_empty_document_yields_nothing():
    assert import_targets("") == []


def test_a_mid_word_at_is_not_an_import():
    assert import_targets("Mail me@example.com about it.\n") == []


def test_an_at_inside_a_code_span_is_not_an_import():
    assert import_targets("Install `@scope/pkg` first.\n") == []


def test_an_import_beside_a_code_span_survives_the_span_being_stripped():
    assert import_targets("`@scope/pkg` and @a.md\n") == ["a.md"]


def test_an_at_inside_a_backtick_fence_is_not_an_import():
    assert import_targets("```\n@a.md\n```\n") == []


def test_an_at_inside_a_tilde_fence_is_not_an_import():
    assert import_targets("~~~\n@a.md\n~~~\n") == []


def test_an_at_on_the_opening_fence_line_is_not_an_import():
    assert import_targets("``` @a.md\nbody\n```\n") == []


def test_scanning_resumes_after_a_fence_closes():
    assert import_targets("```\n@a.md\n```\n@b.md\n") == ["b.md"]


def test_an_indented_fence_still_opens_a_block():
    assert import_targets("   ```\n@a.md\n   ```\n") == []
