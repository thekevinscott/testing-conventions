"""Unit tests for parsing the adapter's ``--lines`` specs."""
from testing_conventions.mutation.parse_line_ranges import parse_line_ranges


def test_specs_group_by_module_in_order():
    assert parse_line_ranges(["calc.py:3-6", "pkg/util.py:1-1", "calc.py:9-10"]) == {
        "calc.py": [(3, 6), (9, 10)],
        "pkg/util.py": [(1, 1)],
    }


def test_the_last_colon_splits_the_path_from_the_range():
    assert parse_line_ranges(["C:/src/calc.py:2-4"]) == {"C:/src/calc.py": [(2, 4)]}


def test_no_specs_is_no_ranges():
    assert parse_line_ranges([]) == {}
