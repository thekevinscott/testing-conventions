"""Colocated unit tests for the `@path` scanner."""
from pathlib import Path

from checks.agents_md_size.resolve_agents_md.imported_paths import imported_paths

CONTAINING = Path("/repo/docs/AGENTS.md")


def test_a_document_with_no_imports_yields_nothing():
    assert imported_paths(CONTAINING, "just prose\n") == []


def test_an_import_resolves_against_the_containing_files_directory():
    assert imported_paths(CONTAINING, "@extra.md\n") == [Path("/repo/docs/extra.md")]


def test_imports_come_back_in_load_order():
    assert imported_paths(CONTAINING, "@a.md\n@b.md\n") == [Path("/repo/docs/a.md"), Path("/repo/docs/b.md")]


def test_two_imports_on_one_line_are_both_found():
    assert imported_paths(CONTAINING, "@a.md @b.md\n") == [Path("/repo/docs/a.md"), Path("/repo/docs/b.md")]


def test_a_parent_relative_import_resolves_upward():
    assert imported_paths(CONTAINING, "@../root.md\n") == [Path("/repo/root.md")]


def test_an_absolute_import_is_left_alone():
    assert imported_paths(CONTAINING, "@/etc/shared.md\n") == [Path("/etc/shared.md")]


def test_a_home_relative_import_expands(monkeypatch):
    monkeypatch.setenv("HOME", "/home/someone")
    assert imported_paths(CONTAINING, "@~/AGENTS.md\n") == [Path("/home/someone/AGENTS.md")]


def test_an_import_inside_a_fenced_block_is_not_an_import():
    assert imported_paths(CONTAINING, "```\n@fenced.md\n```\n") == []


def test_an_import_after_a_closed_fence_is_still_found():
    assert imported_paths(CONTAINING, "```\n@fenced.md\n```\n@after.md\n") == [Path("/repo/docs/after.md")]


def test_a_tilde_fence_also_opens_a_block():
    assert imported_paths(CONTAINING, "~~~\n@fenced.md\n~~~\n@after.md\n") == [Path("/repo/docs/after.md")]


def test_an_import_inside_a_code_span_is_not_an_import():
    assert imported_paths(CONTAINING, "write `@spanned.md` like so\n") == []


def test_a_code_span_does_not_hide_the_rest_of_its_line():
    assert imported_paths(CONTAINING, "`@spanned.md` @real.md\n") == [Path("/repo/docs/real.md")]


def test_a_mid_word_at_sign_is_not_an_import():
    assert imported_paths(CONTAINING, "me@thekevinscott.com\n") == []
