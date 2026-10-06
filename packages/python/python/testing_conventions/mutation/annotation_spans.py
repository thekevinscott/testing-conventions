"""The source spans a module's type annotations occupy.

An annotation holds type metadata the interpreter need never evaluate, so a mutation inside one
runs no code and no test can fail on it; `docs/explanation/mutation.md` carries the reasoning.
"""
from __future__ import annotations

import ast

# Each annotation-bearing node kind and the attribute holding the annotation: a parameter's and an
# annotated assignment's `annotation`, a function's `returns`, and a PEP 695 `type` alias's
# `value`. `ast.TypeAlias` arrived in 3.12, and `isinstance(node, ())` is False where it is absent.
ANNOTATION_ATTRIBUTES = (
    (ast.arg, "annotation"),
    (ast.AnnAssign, "annotation"),
    ((ast.FunctionDef, ast.AsyncFunctionDef), "returns"),
    (getattr(ast, "TypeAlias", ()), "value"),
)


def annotation_spans(source):
    """Every annotation in ``source`` as an end-exclusive ``((line, column), (line, column))``
    span. Columns count characters, the convention the mutation engine's positions use, where
    ``ast`` counts utf-8 bytes. Source the interpreter cannot parse yields no spans, so a mutant
    there stays a survivor rather than vanishing silently."""
    try:
        tree = ast.parse(source)
    except SyntaxError:
        return ()
    lines = source.splitlines()

    def position(line, byte_column):
        return line, len(lines[line - 1].encode("utf-8")[:byte_column].decode("utf-8"))

    def annotations():
        for node in ast.walk(tree):
            for kinds, attribute in ANNOTATION_ATTRIBUTES:
                annotation = getattr(node, attribute) if isinstance(node, kinds) else None
                if annotation is not None:
                    yield annotation

    return tuple(
        (
            position(annotation.lineno, annotation.col_offset),
            position(annotation.end_lineno, annotation.end_col_offset),
        )
        for annotation in annotations()
    )
