"""Colocated unit tests for instructions-file discovery (pure — a list in, a list out)."""
from checks.agents_md_size.instructions_files import instructions_files


def test_an_agents_md_at_the_root_is_an_instructions_file():
    assert instructions_files(["AGENTS.md"]) == ["AGENTS.md"]


def test_a_claude_md_is_an_instructions_file():
    assert instructions_files(["CLAUDE.md"]) == ["CLAUDE.md"]


def test_a_nested_instructions_file_is_found_at_any_depth():
    assert instructions_files(["a/b/c/AGENTS.md"]) == ["a/b/c/AGENTS.md"]


def test_an_ordinary_markdown_file_is_not_one():
    assert instructions_files(["README.md", "docs/guide.md"]) == []


def test_the_match_is_on_the_whole_basename_not_a_suffix():
    # `MY-AGENTS.md` is somebody's notes, not a file an agent loads.
    assert instructions_files(["MY-AGENTS.md", "notCLAUDE.md"]) == []


def test_the_name_is_case_sensitive():
    # Only the exact spelling is loaded; `agents.md` is a different file.
    assert instructions_files(["agents.md", "claude.MD"]) == []


def test_the_result_is_sorted():
    assert instructions_files(["z/AGENTS.md", "a/CLAUDE.md"]) == ["a/CLAUDE.md", "z/AGENTS.md"]


def test_an_empty_tree_yields_nothing():
    assert instructions_files([]) == []
