"""Unit tests for the annotation spans an interpreter never evaluates."""
import sys

import pytest

from testing_conventions.mutation.annotation_spans import annotation_spans

SIGNATURE = "def add(a: int | None, b: int = 10 - 4) -> int | None:\n    return a\n"


def test_a_signatures_parameter_and_return_annotations_each_get_a_span():
    # `int | None` at columns 11 and 43, `int` at 26 — and `10 - 4`, the default, is live code
    # with no span of its own.
    assert sorted(annotation_spans(SIGNATURE)) == [
        ((1, 11), (1, 21)),
        ((1, 26), (1, 29)),
        ((1, 43), (1, 53)),
    ]


def test_an_annotated_assignments_span_covers_a_nested_union_but_not_the_value():
    source = 'TOTALS: dict[str, int | None] = {"total": 11 - 4}\n'
    assert annotation_spans(source) == (((1, 8), (1, 29)),)


def test_columns_count_characters_where_ast_counts_utf8_bytes():
    # `ñ` is two bytes and one character, so `ast` puts the annotation at byte column 10 while
    # the mutation engine's positions count the 9 characters before it.
    assert annotation_spans("def f(ñ: int) -> None:\n    return None\n") == (
        ((1, 17), (1, 21)),
        ((1, 9), (1, 12)),
    )


def test_an_async_functions_return_annotation_gets_a_span():
    assert annotation_spans("async def f() -> int | None:\n    return None\n") == (
        ((1, 17), (1, 27)),
    )


def test_a_span_runs_across_the_lines_a_parenthesized_annotation_spreads_over():
    source = "def f() -> (\n    int\n    | None\n):\n    return None\n"
    assert annotation_spans(source) == (((2, 4), (3, 10)),)


@pytest.mark.skipif(sys.version_info < (3, 12), reason="PEP 695 `type` arrived in 3.12")
def test_a_type_aliases_value_gets_a_span():
    assert annotation_spans("type Alias = int | None\n") == (((1, 13), (1, 23)),)


def test_source_the_interpreter_cannot_parse_has_no_spans():
    # Fail closed: a mutant in unparseable source stays a survivor rather than vanishing.
    assert annotation_spans("def add(a: int\n") == ()


def test_a_parameter_without_an_annotation_has_no_span():
    assert annotation_spans("def add(a, b):\n    return a + b\n") == ()


def test_a_later_lines_columns_are_converted_against_that_line():
    # The `ñ` sits on line 2, so a conversion that measured line 1 would place `int` early.
    assert annotation_spans("def f(\n    ñ: int,\n):\n    return None\n") == (((2, 7), (2, 10)),)
