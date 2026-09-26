"""Colocated unit tests for import-target resolution."""
from checks.agents_md_size.resolve_target import resolve_target


def test_a_target_resolves_against_the_containing_files_directory():
    assert resolve_target("pkg/AGENTS.md", "style.md") == "pkg/style.md"


def test_a_target_beside_a_root_level_file_keeps_no_directory_prefix():
    assert resolve_target("AGENTS.md", "agents/style.md") == "agents/style.md"


def test_an_explicit_dot_slash_target_normalizes_away():
    assert resolve_target("pkg/AGENTS.md", "./style.md") == "pkg/style.md"


def test_a_target_climbing_back_into_the_tree_resolves():
    assert resolve_target("pkg/docs/AGENTS.md", "../style.md") == "pkg/style.md"


def test_a_target_climbing_above_the_root_is_outside_the_checkout():
    assert resolve_target("AGENTS.md", "../style.md") is None


def test_a_home_relative_target_is_outside_the_checkout():
    assert resolve_target("AGENTS.md", "~/.claude/AGENTS.md") is None


def test_an_absolute_target_is_outside_the_checkout():
    assert resolve_target("AGENTS.md", "/etc/AGENTS.md") is None


def test_a_filename_merely_beginning_with_dots_is_inside_the_checkout():
    assert resolve_target("AGENTS.md", "..style.md") == "..style.md"
