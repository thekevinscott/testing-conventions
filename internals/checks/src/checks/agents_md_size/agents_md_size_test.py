"""Colocated unit tests for the budget check."""
from pathlib import Path
from typing import get_type_hints

from checks.agents_md_size.agents_md_size import agents_md_size

MAX_LEN = 10


def test_it_takes_a_path_written_either_way():
    # Python 3.14 defers annotation evaluation, so without this the signature is never evaluated at
    # all — a typo in it would go unnoticed and the mutation gate reports the union as unkillable.
    assert get_type_hints(agents_md_size)["path_to_agents_md"] == str | Path


def test_a_file_under_the_budget_fits(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("x" * (MAX_LEN - 1), encoding="utf-8")
    assert agents_md_size(entry, MAX_LEN) is True


def test_a_file_exactly_at_the_budget_does_not_fit(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("x" * MAX_LEN, encoding="utf-8")
    assert agents_md_size(entry, MAX_LEN) is False


def test_a_file_one_character_past_the_budget_does_not_fit(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("x" * (MAX_LEN + 1), encoding="utf-8")
    assert agents_md_size(entry, MAX_LEN) is False


def test_an_import_counts_against_the_budget(tmp_path):
    entry = tmp_path / "AGENTS.md"
    entry.write_text("@extra.md", encoding="utf-8")
    (tmp_path / "extra.md").write_text("x" * MAX_LEN, encoding="utf-8")
    assert agents_md_size(entry, MAX_LEN) is False


def test_the_budget_is_measured_in_characters_not_bytes(tmp_path):
    entry = tmp_path / "AGENTS.md"
    # Two characters, four UTF-8 bytes. A byte count would call this over a budget of three.
    entry.write_text("é" * 2, encoding="utf-8")
    assert agents_md_size(entry, 3) is True
