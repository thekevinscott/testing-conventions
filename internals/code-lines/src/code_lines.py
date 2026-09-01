#!/usr/bin/env python3
"""Report non-test code lines per source area — see docs/internals/repo.md."""
from __future__ import annotations

import ast
import io
import re
import subprocess
import tokenize
from collections import Counter
from pathlib import Path, PurePosixPath

SOURCE_SUFFIXES = {".rs", ".py", ".ts", ".js", ".mjs"}
SOURCE_TREES = ("packages", "internals")
AREA_DEPTH = 3
CFG_TEST = "#[cfg(test)]"
RUST_RAW_STRING = re.compile(r'b?r(#*)"')
PY_NON_CODE = frozenset({tokenize.COMMENT, tokenize.NL, tokenize.NEWLINE,
                         tokenize.INDENT, tokenize.DEDENT, tokenize.ENDMARKER})
CodeFlags = list[bool]
CommentFlags = list[bool]
Regions = list[tuple[int, int]]
Scan = tuple[CodeFlags, CommentFlags, Regions]


def area_of(path: str) -> str | None:
    """The source area `path` belongs to, or `None` when it is out of scope."""
    parts = PurePosixPath(path).parts
    if len(parts) <= AREA_DEPTH or parts[0] not in SOURCE_TREES:
        return None
    if PurePosixPath(path).suffix not in SOURCE_SUFFIXES:
        return None
    return "/".join(parts[:AREA_DEPTH])


def is_test_file(path: str) -> bool:
    """True for test code, wherever it lives."""
    name = PurePosixPath(path).name
    if "tests" in PurePosixPath(path).parts:
        return True
    if name.endswith((".test.ts", ".test.js", ".test.mjs")):
        return True
    return name.endswith("_test.py") or (name.startswith("test_") and name.endswith(".py"))


def docstring_starts(text: str) -> set[tuple[int, int]]:
    """The `(row, column)` of every module, class and function docstring in `text`."""
    holders = (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)
    starts = set()
    for node in ast.walk(ast.parse(text)):
        if not isinstance(node, holders) or not node.body:
            continue
        first = node.body[0]
        if isinstance(first, ast.Expr) and isinstance(first.value, ast.Constant):
            if isinstance(first.value.value, str):
                starts.add((first.value.lineno, first.value.col_offset))
    return starts


def scan_python(text: str, total: int) -> Scan:
    """Mark code and comment lines in Python source; docstrings count as comments."""
    code = [False] * total
    comment = [False] * total
    docstrings = docstring_starts(text)
    for token in tokenize.generate_tokens(io.StringIO(text).readline):
        if token.type == tokenize.COMMENT:
            marks = comment
        elif token.type in PY_NON_CODE:
            continue
        elif token.type == tokenize.STRING and token.start in docstrings:
            marks = comment
        else:
            marks = code
        for row in range(token.start[0] - 1, min(token.end[0], total)):
            marks[row] = True
    return code, comment, []


def scan_clike(text: str, total: int, *, rust: bool) -> Scan:
    """Mark code and comment lines, plus Rust's inline `#[cfg(test)]` regions, in C-like source."""
    code = [False] * total
    comment = [False] * total
    regions: list[tuple[int, int]] = []
    row = 0
    i = 0
    end = len(text)
    region_start: int | None = None
    depth = 0
    opened = False

    def mark(marks: list[bool]) -> None:
        if row < total:
            marks[row] = True

    def consume_quoted(i: int, closing: str, escapes: bool) -> int:
        """Advance past a string, char or template literal, counting the rows it spans."""
        nonlocal row
        while i < end and not text.startswith(closing, i):
            if escapes and text[i] == "\\":
                i += 1
            if i < end and text[i] == "\n":
                row += 1
                mark(code)
            i += 1
        return i + len(closing)

    while i < end:
        char = text[i]
        if char == "\n":
            row += 1
            i += 1
            continue
        if char.isspace():
            i += 1
            continue
        if text.startswith("//", i):
            mark(comment)
            newline = text.find("\n", i)
            i = end if newline < 0 else newline
            continue
        if text.startswith("/*", i):
            mark(comment)
            nesting = 1
            i += 2
            while i < end and nesting:
                if rust and text.startswith("/*", i):
                    nesting += 1
                    i += 2
                elif text.startswith("*/", i):
                    nesting -= 1
                    i += 2
                else:
                    if text[i] == "\n":
                        row += 1
                        mark(comment)
                    i += 1
            continue
        if rust:
            raw = RUST_RAW_STRING.match(text, i)
            if raw:
                mark(code)
                i = consume_quoted(raw.end(), '"' + raw.group(1), escapes=False)
                continue
            if char == "'":
                mark(code)
                # Reading the lifetime `'a` as a char literal swallows the rest of the file.
                literal = text[i + 1 : i + 2] == "\\" or text[i + 2 : i + 3] == "'"
                i = consume_quoted(i + 1, "'", escapes=True) if literal else i + 1
                continue
            if region_start is None and text.startswith(CFG_TEST, i):
                mark(code)
                region_start, depth, opened = row, 0, False
                i += len(CFG_TEST)
                continue
        if char == '"' or (not rust and char in "`'"):
            mark(code)
            i = consume_quoted(i + 1, char, escapes=True)
            continue
        if region_start is not None and char in "{}":
            depth += 1 if char == "{" else -1
            opened = opened or char == "{"
            if opened and depth == 0:
                regions.append((region_start, row))
                region_start = None
        mark(code)
        i += 1
    return code, comment, regions


def classify(path: str, text: str) -> list[str]:
    """The kind of every line in `text`: code, comment, blank or test."""
    lines = text.splitlines()
    if is_test_file(path):
        return ["test"] * len(lines)
    if path.endswith(".py"):
        code, comment, regions = scan_python(text, len(lines))
    else:
        code, comment, regions = scan_clike(text, len(lines), rust=path.endswith(".rs"))
    kinds = []
    excluded = {row for first, last in regions for row in range(first, last + 1)}
    for row, line in enumerate(lines):
        if row in excluded:
            kinds.append("test")
        elif not line.strip():
            kinds.append("blank")
        elif code[row]:
            kinds.append("code")
        else:
            kinds.append("comment")
    return kinds


def tally(files: dict[str, str]) -> dict[str, Counter]:
    """Line kinds per area, over a mapping of in-scope path to source text."""
    areas: dict[str, Counter] = {}
    for path, text in sorted(files.items()):
        area = area_of(path)
        if area is None:
            continue
        areas.setdefault(area, Counter()).update(classify(path, text))
    return areas


def render(areas: dict[str, Counter]) -> str:
    """The markdown table, sorted by code descending, with a totals row."""
    columns = ("code", "comment", "blank", "test")
    rows = sorted(areas.items(), key=lambda entry: (-entry[1]["code"], entry[0]))
    total = Counter()
    for _, counts in rows:
        total.update(counts)
    lines = ["| area | code | comment | blank | test |", "| --- | ---: | ---: | ---: | ---: |"]
    for area, counts in rows:
        lines.append(f"| {area} | " + " | ".join(str(counts[c]) for c in columns) + " |")
    tail = " | ".join(str(total[c]) for c in columns[1:])
    lines.append(f"| **total** | **{total['code']}** | {tail} |")
    return "\n".join(lines)


def repo_root() -> Path:
    """The root of the repository the current directory sits in."""
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True
    )
    return Path(out.stdout.strip())


def tracked_paths(root: Path) -> list[str]:
    """Every tracked file under the source trees, as repo-relative POSIX paths."""
    out = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", *SOURCE_TREES],
        capture_output=True,
        text=True,
        check=True,
    )
    return [path for path in out.stdout.split("\0") if path]


def read_source(root: Path, path: str) -> str:
    return (root / path).read_text(encoding="utf-8")


def collect(root: Path) -> dict[str, str]:
    """Read every in-scope file under `root`."""
    return {
        path: read_source(root, path) for path in tracked_paths(root) if area_of(path) is not None
    }


def main() -> int:
    root = repo_root()
    print(render(tally(collect(root))))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
