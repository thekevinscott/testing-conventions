"""Colocated unit tests for the budget comparison."""
from checks.agents_md_size.agents_md_size import agents_md_size

MAX_LEN = 10


def test_content_shorter_than_the_budget_is_not_over():
    assert agents_md_size("x" * (MAX_LEN - 1), MAX_LEN) is False


def test_content_exactly_at_the_budget_is_not_over():
    assert agents_md_size("x" * MAX_LEN, MAX_LEN) is False


def test_content_one_character_past_the_budget_is_over():
    assert agents_md_size("x" * (MAX_LEN + 1), MAX_LEN) is True


def test_empty_content_is_not_over():
    assert agents_md_size("", MAX_LEN) is False


def test_a_zero_budget_makes_any_content_over():
    assert agents_md_size("x", 0) is True


def test_a_zero_budget_still_passes_empty_content():
    assert agents_md_size("", 0) is False


def test_the_budget_is_measured_in_characters_not_bytes():
    # Two characters, four UTF-8 bytes. A byte count would call this over a budget of three.
    assert agents_md_size("é" * 2, 3) is False
