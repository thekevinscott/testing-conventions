"""Colocated unit tests for the size budgets.

The defaults are the gate's whole contract with a repo that passes no options, so they are pinned
here rather than left to whatever the dataclass happens to say.
"""
from checks.agents_md_size.limits import Limits


def test_the_soft_budget_is_anthropics_stated_line_target():
    assert Limits.warn_lines == 200


def test_the_hard_line_budget_is_twice_the_soft_one():
    assert Limits.max_lines == 2 * Limits.warn_lines


def test_the_hard_byte_budget_is_codexs_truncation_cliff():
    assert Limits.max_bytes == 32768


def test_the_soft_byte_budget_is_half_the_hard_one():
    assert Limits.warn_bytes * 2 == Limits.max_bytes


def test_each_byte_budget_is_its_line_budget_at_seventy_five_bytes_a_line():
    assert Limits.warn_bytes >= Limits.warn_lines * 75
    assert Limits.max_bytes >= Limits.max_lines * 75


def test_the_budgets_are_overridable_per_run():
    limits = Limits(warn_lines=1, warn_bytes=2, max_lines=3, max_bytes=4)
    assert (limits.warn_lines, limits.warn_bytes, limits.max_lines, limits.max_bytes) == (1, 2, 3, 4)
