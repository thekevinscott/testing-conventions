"""Colocated unit tests for the `@path` import scan and its resolution against the importer."""
from checks.agents_md_size.imports import imported_paths


def _at_root(text):
    """Scan a document that sits at the repo root, where a target resolves to itself."""
    return imported_paths("AGENTS.md", text)


def test_a_bare_at_path_on_its_own_line_is_an_import():
    assert _at_root("@agents/style.md\n") == ["agents/style.md"]


def test_an_import_after_prose_on_the_same_line_is_found():
    assert _at_root("See @agents/style.md for details.\n") == ["agents/style.md"]


def test_every_import_is_returned_in_document_order():
    assert _at_root("@a.md\n@b.md\n") == ["a.md", "b.md"]


def test_a_repeated_import_is_returned_once_per_occurrence():
    assert _at_root("@a.md @a.md\n") == ["a.md", "a.md"]


def test_a_document_with_no_imports_yields_nothing():
    assert _at_root("# Title\n\nProse.\n") == []


def test_an_empty_document_yields_nothing():
    assert _at_root("") == []


def test_a_mid_word_at_is_not_an_import():
    assert _at_root("Mail me@example.com about it.\n") == []


def test_an_at_inside_a_code_span_is_not_an_import():
    assert _at_root("Install `@scope/pkg` first.\n") == []


def test_an_import_beside_a_code_span_survives_the_span_being_stripped():
    assert _at_root("`@scope/pkg` and @a.md\n") == ["a.md"]


def test_an_at_inside_a_backtick_fence_is_not_an_import():
    assert _at_root("```\n@a.md\n```\n") == []


def test_an_at_inside_a_tilde_fence_is_not_an_import():
    assert _at_root("~~~\n@a.md\n~~~\n") == []


def test_an_at_on_the_opening_fence_line_is_not_an_import():
    assert _at_root("``` @a.md\nbody\n```\n") == []


def test_scanning_resumes_after_a_fence_closes():
    assert _at_root("```\n@a.md\n```\n@b.md\n") == ["b.md"]


def test_an_indented_fence_still_opens_a_block():
    assert _at_root("   ```\n@a.md\n   ```\n") == []


def test_a_target_resolves_against_the_importing_files_own_directory():
    assert imported_paths("pkg/AGENTS.md", "@style.md\n") == ["pkg/style.md"]


def test_a_dot_slash_target_resolves_to_the_same_path():
    assert imported_paths("pkg/AGENTS.md", "@./style.md\n") == ["pkg/style.md"]


def test_an_upward_target_that_stays_inside_the_checkout_is_kept():
    assert imported_paths("pkg/deep/AGENTS.md", "@../style.md\n") == ["pkg/style.md"]


def test_an_upward_target_that_escapes_the_checkout_is_dropped():
    assert imported_paths("AGENTS.md", "@../outside.md\n") == []


def test_a_home_relative_target_is_dropped():
    assert imported_paths("AGENTS.md", "@~/dotfiles/AGENTS.md\n") == []


def test_an_absolute_target_is_dropped():
    assert imported_paths("AGENTS.md", "@/etc/AGENTS.md\n") == []


def test_a_dropped_target_does_not_stop_the_scan():
    assert imported_paths("AGENTS.md", "@~/outside.md\n@inside.md\n") == ["inside.md"]


def test_a_dropped_target_does_not_stop_the_rest_of_its_own_line():
    # Same line, so the skip has to resume the match loop rather than leave it.
    assert imported_paths("AGENTS.md", "@~/outside.md @inside.md\n") == ["inside.md"]
