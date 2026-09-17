"""Colocated unit tests for the path-length-gate budget decision."""
from checks.path_length_gate.decide import BUDGET, over_budget


def test_a_path_at_the_budget_passes():
    assert over_budget(["a" * BUDGET]) == []


def test_a_path_one_over_the_budget_fails():
    path = "a" * (BUDGET + 1)
    assert over_budget([path]) == [(path, BUDGET + 1)]


def test_violations_sort_longest_first():
    shorter_over = "a" * (BUDGET + 1)
    longer_over = "b" * (BUDGET + 5)
    assert over_budget([shorter_over, longer_over]) == [
        (longer_over, BUDGET + 5),
        (shorter_over, BUDGET + 1),
    ]


def test_ties_sort_alphabetically():
    a = "a" * (BUDGET + 1)
    b = "b" * (BUDGET + 1)
    assert over_budget([b, a]) == [(a, BUDGET + 1), (b, BUDGET + 1)]


def test_a_custom_budget_is_honored():
    assert over_budget(["abcdef"], budget=5) == [("abcdef", 6)]


def test_paths_within_a_custom_budget_pass():
    assert over_budget(["abcde"], budget=5) == []
