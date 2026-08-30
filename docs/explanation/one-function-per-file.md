---
description: Why a file holds one substantial function — and why the line threshold that decides "substantial" is yours to set.
---

# One function per file

`unit one-function-per-file` puts a ceiling on how much a single source file can hold: **one
module-scope function whose body runs longer than the configured threshold**. Trivial functions —
the ones at or under the threshold — sit together freely. This page explains why the standard
draws that line and why the threshold is configuration rather than a constant.

## Why the file is the unit

[Colocation](./colocated-test) pairs `widget.py` with `widget_test.py` and makes the unit boundary
structural. That pairing is worth exactly as much as the file is coherent. A file holding ten
functions still has one colocated test, so the 1:1 mapping the presence check enforces describes
the *file*, not the code in it: the test file's name announces `widget`, and a reader has to open
it to learn which of the ten functions it covers.

One function per file collapses that gap. The file name **is** the subject, so the colocated test
names its subject too, a rename moves both halves together, and the diff for a behavior change
touches the one function that changed. The rungs above presence sharpen the same way — a
[coverage](./coverage) or [mutation](./mutation) result reported against a file is a result about
one function, so a survivor points at the code that produced it.

The rule also gives an agent a structural stopping point. "Add the helper here" is the cheapest
move in an agentic edit, and it is how a 40-line module becomes a 400-line one with a single
sprawling test beside it. A gate on the count makes the next function a new file.

## Why a threshold, and why it's yours

A one-line function is an expression with a name — a predicate, a formatter, a default. It carries
no branch to test in isolation, and giving it a file of its own is ceremony, not structure. So the
rule counts only functions whose body runs longer than **`max_lines`**, and the default is `1`:
one line is trivial, anything longer earns its own module.

That default is the standard's opinion, not a law of the domain. A codebase whose natural grain is
small functions may want the line at three or five; one adopting the rule onto an existing tree may
start at twenty and walk it down. `max_lines` is a plain number in the config, per language:

```toml
[python]
one_function_per_file = { max_lines = 5 }
```

Raising it never turns the rule off — a file with two twenty-line functions fails at
`max_lines = 5` and at `max_lines = 19` alike. It moves only the boundary between "trivial enough
to share" and "substantial enough to own the file".

## What counts as a line

A function's length is the count of **code** lines in its body. Blank lines and comments don't
count, and neither does a Python docstring: documentation is the same documentation whether the
language spells it `#`, `//`, `///`, or a string on the first line, and it should never be what
pushes a function over the line.

The signature doesn't count either — only the body — so a function with six named parameters
across six lines is still a one-line function if its body is one line.

## What counts as a function

**Module-scope functions**, in each language's own spelling:

- **Python** — `def` and `async def` at module level.
- **TypeScript** — top-level `function` declarations, and a top-level `const` / `let` / `var` bound
  to an arrow function or function expression. `export` makes no difference either way.
- **Rust** — `fn` items at the top level of the file.

Methods are not module-scope functions: a Python class's methods, a TypeScript class's methods, and
a Rust `impl` block's methods all belong to the type that owns them, and the type is the file's one
subject. Nested functions and callbacks aren't module-scope either — a closure passed to `map`, a
handler defined inside another function, and a Rust `fn` declared inside a function body are all
part of their enclosing function's body, counted in its length rather than as functions of their
own.

Unit tests are never subjects. The scan reads the same source tree
[colocated-test](./colocated-test) does, so Python and TypeScript test files, Rust's inline
`#[cfg(test)]` modules, and the suite directories beside the package are all outside it — a test
file is *supposed* to hold a function per case.

## When the rule is wrong

A file whose functions genuinely belong together — a generated module, a table of small
constructors, a public surface re-exported as one API — takes a
[`one-function-per-file` exemption](/guide/configure#exempt-a-file) with a reason, like every other
deliberate omission in the standard.
