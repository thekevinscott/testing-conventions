"""Colocated unit tests for the closure size verdict."""
from types import SimpleNamespace

from checks.agents_md_size.assess import assess

# A stand-in for the real budgets, small enough to name a boundary in a literal and far enough
# apart that a case straddling one axis stays clear of the other.
LIMITS = SimpleNamespace(warn_lines=10, warn_bytes=1000, max_lines=20, max_bytes=2000)


def _members(lines=0, width=1, count=1):
    """`count` files of `lines` lines each, every line `width` characters wide."""
    text = "".join(f"{'x' * width}\n" for _ in range(lines))
    return [(f"f{index}.md", text) for index in range(count)]


def test_an_empty_closure_measures_zero():
    assert assess([], LIMITS) == (0, 0, "ok")


def test_a_small_closure_is_ok():
    assert assess(_members(lines=3), LIMITS) == (3, 6, "ok")


def test_lines_and_bytes_sum_across_every_member():
    assert assess(_members(lines=3, count=2), LIMITS)[:2] == (6, 12)


def test_a_multibyte_character_is_counted_in_utf_8_bytes():
    assert assess([("a.md", "é\n")], LIMITS)[1] == 3


def test_a_final_line_without_a_newline_still_counts():
    assert assess([("a.md", "one\ntwo")], LIMITS)[0] == 2


def test_a_closure_at_the_soft_line_budget_is_ok():
    assert assess(_members(lines=LIMITS.warn_lines), LIMITS)[2] == "ok"


def test_a_closure_one_line_past_the_soft_budget_warns():
    assert assess(_members(lines=LIMITS.warn_lines + 1), LIMITS)[2] == "warn"


def test_a_closure_at_the_hard_line_budget_only_warns():
    assert assess(_members(lines=LIMITS.max_lines), LIMITS)[2] == "warn"


def test_a_closure_one_line_past_the_hard_budget_fails():
    assert assess(_members(lines=LIMITS.max_lines + 1), LIMITS)[2] == "fail"


def test_a_closure_at_the_soft_byte_budget_is_ok():
    assert assess([("a.md", "x" * LIMITS.warn_bytes)], LIMITS)[2] == "ok"


def test_a_closure_one_byte_past_the_soft_budget_warns():
    assert assess([("a.md", "x" * (LIMITS.warn_bytes + 1))], LIMITS)[2] == "warn"


def test_a_closure_at_the_hard_byte_budget_only_warns():
    assert assess([("a.md", "x" * LIMITS.max_bytes)], LIMITS)[2] == "warn"


def test_a_closure_one_byte_past_the_hard_budget_fails():
    assert assess([("a.md", "x" * (LIMITS.max_bytes + 1))], LIMITS)[2] == "fail"


def test_bytes_alone_can_fail_a_closure_well_under_the_line_budget():
    assert assess(_members(lines=2, width=LIMITS.max_bytes), LIMITS)[2] == "fail"


def test_lines_alone_can_fail_a_closure_well_under_the_byte_budget():
    assert assess(_members(lines=LIMITS.max_lines + 1), LIMITS)[2] == "fail"


def test_bytes_alone_can_warn_a_closure_well_under_the_line_budget():
    assert assess(_members(lines=1, width=LIMITS.warn_bytes + 1), LIMITS)[2] == "warn"


def test_lines_alone_can_warn_a_closure_well_under_the_byte_budget():
    assert assess(_members(lines=LIMITS.warn_lines + 1), LIMITS)[2] == "warn"
