---
description: Why every source file carries a colocated, matching-named unit test — and why a changed source must change its test with it.
---

# Colocated tests

`unit colocated-test` is the first rung of the [unit ladder](./#the-unit-ladder-exist-run-verify):
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

The check is **tree-wide presence**: every source file under the scan root has its colocated,
matching-named unit test.

- **Python** — `foo.py` → `foo_test.py`, side by side. `__init__.py` is not special: an empty one
  is skipped (no logic), a non-empty one needs a test or an
  [exemption](../guide/configure#exempt-a-file).
- **TypeScript** — `foo.ts` / `.tsx` / `.mts` / `.cts` → a colocated `foo.test.*` of the matching
  extension. Declaration files (`*.d.ts`) carry no runtime code and are ignored.
- **Rust** — units are inline `#[cfg(test)]` modules, not sibling files, so the check is presence
  of the inline module: a `src` file that defines a function with a body but has no `#[cfg(test)]`
  module is an orphan. A test module is one gated by a positively-required `test` — `#[cfg(test)]`
  or `#[cfg(all(test, …))]`; a `#[cfg(not(test))]` module compiles in *non-test* builds, so it is
  production code and counts as behavior to test, never as the inline test. Module-declaration
  files (only `mod` / `use`) and type-only files (no `fn`) aren't subjects; `tests/`, `benches/`,
  `examples/`, and `build.rs` are skipped.

Empty or comment-only files are never subjects, and a file with a `colocated-test`
[exemption](../guide/configure#exempt-a-file) is deliberately omitted, with a reason.

## Co-change: a stale test is an invisible orphan

Presence isn't enough on a pull request. A source edit that leaves the colocated test untouched
lets the test silently go stale — it still *exists*, but it pins the old behavior. So on pull
requests the check also runs **commit-scoped** over the `<base>...HEAD` diff (Python, TypeScript):

- a **modified** source must have its colocated test in the diff too;
- a **deleted** source that had a test in the base tree must delete or update that test with it;
- an **added** source is not a subject — brand-new code is the [coverage floor](./coverage)'s
  concern.

Changing a test on its own is always fine. Rust units are inline in the same file, so a sibling
test can't go stale and the co-change check doesn't apply. A `co-change`
[exemption](../guide/configure#exempt-a-file) lifts the check for a file, independently of the
presence exemption.

Co-change and [changed-line coverage](./coverage#the-changed-line-floor) are complementary:
co-change enforces that the source and its *test* move together; the coverage floor enforces that
the changed *lines* are exercised. One can pass while the other fails.
