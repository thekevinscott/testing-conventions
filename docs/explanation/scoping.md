---
description: How the scan is scoped and how a deliberate omission is recorded — path scopes, exemptions name, and there is no ignore-glob.
---

# Scoping and exemptions

Every check needs two answers: *which files are subjects*, and *how a deliberate omission is
recorded*. The standard's answers are deliberately narrow — `path` scopes the scan, and a named,
reason-required exemption records the omission. This page explains why those are the only two
mechanisms.

## `path` scopes the scan

The workflow's `path` input (and the CLI's `<PATH>` argument) is the scan root: the unit-tier
checks recursively take **every file under it** as a subject. Scoping happens by pointing `path`
at the real source directory — `src` for a single-package repo, each package's own source dir in
a [monorepo](../monorepo) — so virtualenvs, tooling, and sibling packages sit *outside* the scan
rather than being filtered out of it.

The suite tiers are derived rather than pointed at. `integration lint` walks up from `path` to
the package root — the nearest directory holding the language's manifest (`pyproject.toml`,
`package.json`, `Cargo.toml`), stopping at the repository boundary — and takes its subjects from
the standard suite directories: `tests/integration/` and `tests/e2e/` (Rust: the crate root's
`tests/`, cargo's own layout). The unit-tier checks leave `<package root>/tests/` to the suites,
so one `path` covers a package whose suites sit beside its sources. A test file under
`<package root>/tests/` outside a standard tier is flagged (`unknown-tier`): the layout is part
of the standard, so a suite the scan would otherwise miss is named as an error. A tree with no
manifest — loose scripts — is scanned at `path` directly, every test file a subject.

That's the whole scoping model. There is no ignore file, no exclude glob, no `.gitignore`
integration: a subject inside the scan root is either checked or *named* as exempt. An ignore-glob
is the failure mode this design refuses — a pattern like `**/generated/**` silently swallows every
future file that happens to match, with no reason attached and no review when a new file slips
under it. The scan root states where the standard applies; the exemption list states, file by
file, where it deliberately doesn't.

## Exemptions: a gate needs a door

A blocking gate with no escape hatch gets disabled. So every check has one — but it's **explicit,
reason-required, and in one file**, never a silent ignore. A launcher shim, a re-export barrel, or
generated code earns a `[[<language>.exempt]]` entry that names the rules it lifts and *why*. The
philosophy is *"zero violations except what you exempted with a reason"* — not *"hit a number you
can soften when it's inconvenient."*

Three properties keep the list honest:

- **Auditable in one diff.** Every exemption lives in the config file, names its rules, and
  carries a reason — so the project's entire omission surface is one reviewable table, unlike
  scattered ignore comments.
- **It can't rot.** An entry's `path` must point to a file that exists; a stale entry is a hard
  error. The list describes the present tree, always.
- **The only automatic exclusions are the empty ones.** A file with no logic (empty or
  comment-only, a `*.d.ts`) has nothing to test and is skipped with no configuration. Everything
  else that should be skipped is skipped *by name, with a reason*.

## Exemptions are line-scoped where it counts

For the measured-line checks — coverage and mutation — a whole-file exemption would be far blunter
than the code it excuses, so it doesn't exist: those entries carry a `lines` list naming the exact
failing lines, and a **determinism guard** rejects any listed line that isn't actually failing.
The exemption is therefore always exactly as big as the code that genuinely can't be tested — you
list the irreducible lines instead of quarantining a whole file around them — and it shrinks
automatically: when the code changes and a listed line starts passing, the guard errors until the
entry is trimmed.

## The bar for exempting

Almost nothing is genuinely untestable. What feels untestable usually needs a technique — inject
the dependency and assert against a fake, drive a framework hook directly, force the dead branch of
a version-conditional import — so the `reason` field should read like the end of an investigation,
naming what was tried, not like a shrug. The mechanics live in
[Configure the rules](../guide/configure#exempt-a-file); the schema in the
[configuration reference](../reference/config#exemptions).
