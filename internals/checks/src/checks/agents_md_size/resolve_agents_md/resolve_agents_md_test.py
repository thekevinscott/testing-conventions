"""Colocated unit tests for import resolution."""
from checks.agents_md_size.resolve_agents_md.resolve_agents_md import resolve_agents_md


def _chain(root, length):
    """A chain of `length` documents, each importing the next; returns the entry.

    Each body marker differs from every filename, so finding `bodyN` means document N was loaded and
    not merely named by the import line in document N-1.
    """
    for hop in range(length):
        target = f"@hop{hop + 1}.md\n" if hop + 1 < length else ""
        (root / f"hop{hop}.md").write_text(f"body{hop}\n{target}", encoding="utf-8")
    return root / "hop0.md"


def test_a_file_with_no_imports_is_its_own_content(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("alone\n", encoding="utf-8")
    assert resolve_agents_md(entry) == "alone\n"


def test_a_missing_entry_resolves_to_nothing(tmp_path):
    assert resolve_agents_md(tmp_path / "absent.md") == ""


def test_an_imported_file_follows_the_entry(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("entry\n@extra.md\n", encoding="utf-8")
    (tmp_path / "extra.md").write_text("extra\n", encoding="utf-8")
    assert resolve_agents_md(entry) == "entry\n@extra.md\nextra\n"


def test_an_import_resolves_against_the_importing_files_directory(tmp_path):
    nested = tmp_path / "nested"
    nested.mkdir()
    entry = tmp_path / "AGENTS.md"
    entry.write_text("@nested/inner.md\n", encoding="utf-8")
    (nested / "inner.md").write_text("@sibling.md\n", encoding="utf-8")
    (nested / "sibling.md").write_text("sibling\n", encoding="utf-8")
    assert resolve_agents_md(entry) == "@nested/inner.md\n@sibling.md\nsibling\n"


def test_a_missing_target_is_skipped(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("@absent.md @present.md\n", encoding="utf-8")
    (tmp_path / "present.md").write_text("present\n", encoding="utf-8")
    assert resolve_agents_md(entry) == "@absent.md @present.md\npresent\n"


def test_an_unreadable_target_is_skipped(tmp_path):
    (tmp_path / "dir.md").mkdir()
    entry = tmp_path / "AGENTS.md"
    entry.write_text("@dir.md @present.md\n", encoding="utf-8")
    (tmp_path / "present.md").write_text("present\n", encoding="utf-8")
    assert resolve_agents_md(entry) == "@dir.md @present.md\npresent\n"


def test_a_target_two_documents_import_is_counted_once(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("@shared.md @other.md\n", encoding="utf-8")
    (tmp_path / "shared.md").write_text("shared\n", encoding="utf-8")
    (tmp_path / "other.md").write_text("@shared.md\n", encoding="utf-8")
    assert resolve_agents_md(entry) == "@shared.md @other.md\nshared\n@shared.md\n"


def test_a_cycle_terminates(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("@loop.md\n", encoding="utf-8")
    (tmp_path / "loop.md").write_text("@AGENTS.md\n", encoding="utf-8")
    assert resolve_agents_md(entry) == "@loop.md\n@AGENTS.md\n"


def test_the_fifth_import_in_a_chain_is_loaded(tmp_path):
    assert "body5" in resolve_agents_md(_chain(tmp_path, 6))


def test_the_sixth_import_in_a_chain_is_not_loaded(tmp_path):
    assert "body6" not in resolve_agents_md(_chain(tmp_path, 7))
