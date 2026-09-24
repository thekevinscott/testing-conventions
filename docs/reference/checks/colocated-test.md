---
description: The colocated-test check — every source file has a colocated, matching-named unit test, and on a pull request a changed source changes its test with it.
---

# `colocated-test`

Every source file under the scan root has a colocated, matching-named unit test — and on a pull
request, a changed source changes its test with it. This page is the complete record of the
check: why it exists, what it enforces per language, when it runs, and its configuration surface.

## Why this check exists

Colocation makes the unit/integration boundary **structural** — by location rather than a tag or
marker: a test that sits next to `widget.py` and is named `widget_test.py` is a unit test of
`widget`, by construction. 1:1 naming makes an orphan visible: when every source file maps to
exactly one test file, a source with no test is caught by a deterministic scan — and so is a test
whose source went away. This is the first rung of the
[unit ladder](/explanation/#the-unit-ladder-exist-→-run-→-verify) — does a test **exist**? — with
[coverage](./unit-coverage) and [mutation](./mutation) asking the stronger questions above it.

The co-change mode exists because presence isn't enough on a pull request: a source edit that
leaves the colocated test untouched lets the test silently go stale — it still exists, but it
pins the old behavior.

## What it enforces

<!--@include: ../../explanation/colocated-test.md#enforces-->

### Co-change, on pull requests

On pull requests the check also runs **commit-scoped** over the `<base>...HEAD` diff (Python and
TypeScript):

- a **modified** source must have its colocated test in the diff too;
- a **deleted** source that had a test in the base tree must delete or update that test with it;
- an **added** source is not a subject — brand-new code is the
  [changed-line coverage floor](./unit-coverage#the-changed-line-job)'s concern.

A modification is a subject when it changes code the compiler sees. The file at the merge base and
the file at `HEAD` are parsed and compared with comments and formatting whitespace normalized away,
so a `#` / `//` / `/* … */` comment edit, a blank-line change, and a trailing-whitespace change
compare equal and pass on their own — the colocated test still pins the behavior the file has. Text
inside a string literal, a Python docstring, and a TypeScript template literal is code, as is the
block structure Python indentation carries, so an edit there is a subject; so is a comment edit
that travels with a code change. Content that fails to parse on either side counts as changed and
is held to its test.

Changing a test on its own always passes. Rust units are inline in the same file, so a sibling
test can't go stale and co-change doesn't apply to Rust — a deliberate asymmetry.

### A Rust binary's entry point

Rust requires a `main` in a binary root, so `src/main.rs` cannot be emptied the way a library module
can. It needs no exemption. Two shapes read as declaration-only; both move the logic into the
library, where a test reaches it.

**Re-export the entry point.** A `pub use` carries no function at all:

```rust
// src/main.rs
pub use mycrate::entrypoint::main;
```

Rust accepts a re-export as the entry point as long as the item is a zero-argument function
returning a `Termination` type.

**Or delegate in a single call.** A `fn main` is declaration-only when its body is one
argument-free call and nothing else:

```rust
// src/main.rs
#[cfg(not(test))]
fn main() -> std::process::ExitCode {
    mycrate::entrypoint::main()
}
```

The shape is deliberately narrow, because anything wider can hide a decision. Exactly one
statement, and that statement a call through a path taking no arguments. A second statement, an
argument, an operator, a `?`, a `match`, a method call — each makes the file a subject again, and
then it needs its own inline `#[cfg(test)]` module.

`#[cfg(not(test))]` is not optional on this shape. Without it the function is instrumented and
reads 0% — `cargo test` replaces a binary's `main` with the harness's own, so no unit test can ever
execute it, and a 100% floor would be unreachable. The re-export needs no such attribute: it emits
no code regions and is absent from the coverage report either way.

That leaves one line outside the unit tier by construction — the process-boundary argv read the
library's `main` performs. Keep it free of decisions, mark it `#[cfg(not(test))]` too, and let
[`e2e-verify`](./e2e-verify) exercise the real binary.

## When it runs

| Mode | Runs | As |
| --- | --- | --- |
| Presence | always, tree-wide | a step of the `Static checks (<language>)` job |
| Co-change | pull requests only (Python, TypeScript), over `<base>...HEAD` | a step of the same job |

The scan covers every file under `source`, leaving `<package root>/tests/` to the suite tiers.
The [`gates` input](/reference/workflow#inputs) names it `colocated-test`; the diff-scoped
co-change mode rides with it.

## Configuration

The check takes no keys of its own. Its exemption names, each a
[`[[<language>.exempt]]` entry](/reference/config#exemptions) with a required `reason`:

| Exemption | Lifts |
| --- | --- |
| `colocated-test` | the presence requirement for one file (a launcher shim, a process entry point) |
| `co-change` | the co-change requirement for one file, independently of presence |

Both exemptions apply to whole files. A declaration-only module is never a subject, and a comment-only or
whitespace-only edit is never a co-change subject — the files themselves decide both, with no
configuration.

## Learn more

- [Explanation — Colocated tests](/explanation/colocated-test): why colocation, and why a stale
  test is an invisible orphan.
- [Respond to a red check](/guide/configure#exempt-a-file): exempting a file, step by step.
