"""Colocated unit tests for the instructions-file filter."""
from checks.agents_md_size.instructions_files import instructions_files


def test_a_root_agents_file_is_an_instructions_file():
    assert instructions_files(["AGENTS.md"]) == ["AGENTS.md"]


def test_a_nested_agents_file_is_an_instructions_file():
    assert instructions_files(["pkg/AGENTS.md"]) == ["pkg/AGENTS.md"]


def test_a_claude_file_is_an_instructions_file():
    assert instructions_files(["CLAUDE.md"]) == ["CLAUDE.md"]


def test_another_markdown_file_is_not():
    assert instructions_files(["README.md", "docs/guide.md"]) == []


def test_a_file_merely_ending_in_the_name_is_not():
    assert instructions_files(["OLD-AGENTS.md", "pkg/my-CLAUDE.md"]) == []


def test_the_result_is_sorted():
    assert instructions_files(["pkg/AGENTS.md", "AGENTS.md"]) == ["AGENTS.md", "pkg/AGENTS.md"]


def test_an_empty_tree_has_no_instructions_files():
    assert instructions_files([]) == []
