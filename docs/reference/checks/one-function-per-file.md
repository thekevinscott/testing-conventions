---
description: The unit one-function-per-file check — a file holds one substantial function; the threshold, what counts as a function, the per-language defaults, and the exemption.
---

# `unit one-function-per-file`

A source file holds at most **one** module-scope function whose body runs longer than the
configured threshold. Functions at or under the threshold are trivial — an expression with a name —
and share a file freely. This page is the complete record of the check.

## Why this check exists

[`unit colocated-test`](./colocated-test) pairs `widget.py` with `widget_test.py`, and that pairing
is worth as much as the file is coherent. A file holding ten functions still has one colocated
test, so the 1:1 mapping describes the *file*, not the code in it. One function per file closes
that gap: the file name is the subject, so a [coverage](./unit-coverage) or [mutation](./mutation)
result reported against a file is a result about one function.

It also gives an agentic edit a structural stopping point. "Add the helper here" is the cheapest
move available, and it is how a 40-line module becomes a 400-line one with a single sprawling test
beside it.

## What it flags

Every module-scope function past the first whose body runs longer than `max_lines`, reported as
`path:line: one-function-per-file — <name> runs N lines, and <holder> already holds this file`.
The first over-threshold function in a file holds it; each later one is a violation naming both.

A function's length is the count of **code** lines in its body — blank lines, comments, and a
Python docstring don't count, and neither does the signature.

What counts as module-scope, per language:

| Language | Subjects |
| --- | --- |
| Python | `def` and `async def` at module level |
| TypeScript | top-level `function` declarations, and a top-level `const` / `let` / `var` bound to an arrow function or function expression, `export` or not |
| Rust | `fn` items at the top level of the file |

Methods, nested functions, and callbacks belong to their owner and are never counted on their own.
The scan reads the same source tree [`unit colocated-test`](./colocated-test) does, so test files,
Rust's inline `#[cfg(test)]` modules, and the suite tiers under `<package root>/tests/` are all
outside it.

## When it runs

Always, as a step of the `Static checks (<language>)` job, for Python, TypeScript, and Rust alike.
It scans the source files under `source`. The [`gates` input](/reference/workflow#inputs) names it
`one-function-per-file`.

**Rust is off until you opt in.** Python and TypeScript run at the default threshold; a Rust run
with no `[rust].one_function_per_file` table reports that the rule is not enabled and passes:

```
unit one-function-per-file: not enabled for rust — set `[rust].one_function_per_file` to opt in
```

In Python and TypeScript a file is a bag of definitions, so "one subject per file" is a choice the
author makes. In Rust a file **is** a module, and grouping a type, its `impl` blocks, and the free
functions around it inside one is how Rust is written. The capability is identical in all three
languages — a Rust tree that wants the rule names a threshold and gets exactly what Python and
TypeScript get. Only the default differs. See
[One function per file](/explanation/one-function-per-file#rust-is-off-until-you-opt-in).

## Configuration

The check takes one key per language table,
[`one_function_per_file`](/reference/config#one-function-per-file), whose `max_lines` sets the
threshold:

| Language | Default |
| --- | --- |
| Python | `max_lines = 1` |
| TypeScript | `max_lines = 1` |
| Rust | off until the table names a threshold |

```toml
[python]
one_function_per_file = { max_lines = 5 }

[rust]
one_function_per_file = { max_lines = 8 }
```

A tree adopting the rule onto existing source starts at a threshold that passes and walks it down;
raising the number never turns the rule off, it only moves the line between "trivial enough to
share" and "substantial enough to own the file".

A file whose functions genuinely belong together takes a `one-function-per-file`
[`[[<language>.exempt]]` entry](/reference/config#exemptions) with a required `reason`:

| Rule | Language | Lifts |
| --- | --- | --- |
| `one-function-per-file` | Python, TypeScript, Rust | the rule for one file (a generated module, a table of small constructors, a public surface re-exported as one API) |

## Learn more

- [Explanation — One function per file](/explanation/one-function-per-file): why the file is the
  unit, and why the threshold is yours to set.
- [Raise the one-function-per-file threshold](/guide/configure#raise-the-one-function-per-file-threshold):
  the config, step by step.
