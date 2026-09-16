---
description: Why every source file carries a colocated, matching-named unit test — and why a changed source must change its test with it.
---

# Colocated tests

`unit colocated-test` is the first rung of the [unit ladder](./#the-unit-ladder-exist-→-run-→-verify):
does a test **exist**? This page explains why the standard pins *where* that test lives and *what
it's named* — and why, on a pull request, a changed source must change its test with it.

## Why colocation

Colocation makes the unit/integration boundary **structural** — by location rather than a tag or
marker. A test that sits next to `widget.py` and is named `widget_test.py` is a unit test of
`widget`, by construction; a test in the integration folder is an integration test. Nothing needs
annotating, and the boundary can't drift, because it's the filesystem.

1:1 naming does the second job: an orphan can't hide. When every source file maps to exactly one
test file, a source with no test is visible to a deterministic check — and so is a test whose
source went away.

## What it enforces

<!-- #region enforces -->
The check is **tree-wide presence**: every source file under the scan root has its colocated,
matching-named unit test.

A source file is a **subject** when it holds a function or control flow anywhere in it — inside a
class body or an object literal included. A function has inputs and a branch depends on something,
so a test can drive either and fail for a reason the source does not already state. A
**declaration-only module** holds neither: it runs once, at import, and importing it is the whole
test. It is skipped. The parser decides this from the file's contents, so no file name is special
and no exemption is involved; a module becomes a subject the moment it gains a `def`, an arrow, a
method, an `if`, or a ternary.

- **Python** — `foo.py` → `foo_test.py`, side by side. A function is `def`, `async def`, or
  `lambda`; control flow is `if`, `for`, `while`, `try`, `with`, `match`, a conditional
  expression, or a comprehension with an `if`. A re-export barrel `__init__.py`, a constants
  module (`TIMEOUT = 30 * 60`, `logger = getLogger(__name__)`), an `Enum`, a dataclass of
  fields, a `TypedDict`, and an annotation-only class are declaration-only. A class is a subject
  through its methods.
- **TypeScript** — `foo.ts` / `.tsx` / `.mts` / `.cts` → a colocated `foo.test.*` of the matching
  extension. A function is a function declaration, a function expression, an arrow, or a method,
  each with a body (a `declare` or overload signature compiles to nothing); control flow adds
  `switch`, `do`, and the ternary. A re-export barrel (`export { x } from './x'`,
  `export * from './x'`), an `export const` of literals, an `as const` object, an enum-only
  module, and a type-only module (`type` / `interface` / `import type` / `export type`) are
  declaration-only. Declaration files (`*.d.ts`) are ignored outright.
- **Rust** — units are inline `#[cfg(test)]` modules, not sibling files, so the check is presence
  of the inline module: a `src` file that defines a function with a body but has no `#[cfg(test)]`
  module is an orphan. A test module is one gated by a positively-required `test` — `#[cfg(test)]`
  or `#[cfg(all(test, …))]`; a `#[cfg(not(test))]` module compiles in *non-test* builds, so it is
  production code and counts as behavior to test, never as the inline test. Module-declaration
  files (only `mod` / `use`) and type-only files (no `fn`) aren't subjects; `tests/`, `benches/`,
  `examples/`, and `build.rs` are skipped.

`and`, `or`, `||`, `&&`, and `??` short-circuit but are not control flow, so
`PORT = env.PORT ?? 3000` stays declaration-only. Empty or comment-only files are never subjects,
and a file with a `colocated-test` [exemption](/guide/configure#exempt-a-file) is deliberately
omitted, with a reason.
<!-- #endregion enforces -->

## Co-change: a stale test is an invisible orphan

Presence isn't enough on a pull request. A source edit that leaves the colocated test untouched
lets the test silently go stale — it still *exists*, but it pins the old behavior. So on pull
requests the check also runs **commit-scoped** over the `<base>...HEAD` diff (Python, TypeScript):

- a **modified** source must have its colocated test in the diff too;
- a **deleted** source that had a test in the base tree must delete or update that test with it;
- an **added** source is not a subject — brand-new code is the [coverage floor](./coverage)'s
  concern.

Co-change reads the same subject definition presence does, from the file's own contents: a
declaration-only module carries no behavior, so editing one is not a stale-test risk and needs no
exemption. A module gains subject status on both halves of the rule together, the moment it adds
a function or control flow.

Changing a test on its own is always fine. Rust units are inline in the same file, so a sibling
test can't go stale and the co-change check doesn't apply. A `co-change`
[exemption](/guide/configure#exempt-a-file) lifts the check for a file, independently of the
presence exemption.

Co-change and [changed-line coverage](./coverage#the-changed-line-floor) are complementary:
co-change enforces that the source and its *test* move together; the coverage floor enforces that
the changed *lines* are exercised. One can pass while the other fails.

### A modification is a change the compiler sees

What makes a modification a subject is the edit itself, not the file's appearance in the diff. The
source at the merge base and the source at `HEAD` are both parsed, with comments and formatting
whitespace normalized away, and the file is a subject when those two forms differ. A comment-only
edit — rewording a `#` note, a `//` line, or a `/* … */` block, or removing one outright — and a
whitespace-only edit — a blank line, trailing spaces — compile to the same program, so the
colocated test still pins the behavior the file has, and the edit passes on its own.

The normalization is deliberately narrow, and everything outside it stays a subject:

- **Python** compares the parser's token stream, where a comment and a blank line never appear.
  A docstring is a string expression, so editing one is a code change, and indentation carries
  block structure, so moving a statement into the block above it is a code change.
- **TypeScript** compares the parsed program with comments removed. Text inside a string or a
  template literal is code, so editing it is a code change.
- A comment edit that travels with a code change is a code change: the code half makes the file a
  subject.
- Content that fails to parse on either side counts as changed, so an unparseable file is held to
  its colocated test.

Both languages reach the same rule through their own parser, so the two arms agree on what an edit
means — the parity the standard holds every rule to.
