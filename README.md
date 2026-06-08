# testing-conventions

**Enforce testing conventions in CI — across languages, from one config.**

`testing-conventions` is an opinionated, config-driven standard that enforces
*where tests live, what counts as a unit test, and whether your coverage number
is telling the truth.* You adopt it the way you adopt a formatter or a linter:
**"this repo follows these conventions,"** checked in CI.

> Coverage measures **execution, not assertion.** A "unit coverage" gate can be
> fully satisfied by *integration* tests whenever the unit/integration boundary
> is unreliable — so the number reads 100% while no real unit tests exist.

> [!NOTE]
> **Status: early development.** These conventions are battle-tested in
> production (see [Why this exists](#why-this-exists)), but the standalone,
> cross-language tool is still being built. This README describes the design and
> the rule set it enforces. Expect things to move.

## Why this exists

The problem is concrete. In one project, a "100% unit coverage" gate stayed
green while the codebase had **no real unit tests at all**:

- The gate counted *any* test not marked `@pytest.mark.integration` as "unit."
- Every test actually lived in the integration directory.
- A feature shipped with only integration tests — and the gate happily reported
  "100% unit coverage."

The number lied, because the unit/integration boundary was defined by a marker
that nothing enforced.

The fix that worked: make the boundary **structural** (defined by *where a test
lives*, not a tag), measure coverage **only on real unit tests**, and **guard
the whole thing in CI** so it can't silently regress. `testing-conventions`
generalizes that fix so any repo, in any language, gets it from one standard
instead of hand-rolled CI scripts.

## What it enforces

These are **hard gates** — static, deterministic checks that fail CI on
violation.

### Test boundary

- **Unit tests are colocated with their source.** `src/**/<name>_test.py`,
  `src/**/*.test.ts`, and so on. Integration tests live in a separate tree
  (`tests/`). Tests in the wrong place fail.
- **The boundary is by location, not markers.** Using a marker or tag (e.g.
  `pytest.mark.integration`) as the unit/integration *selector* is banned —
  that's the exact failure mode this tool exists to prevent.
- **Each unit test maps 1:1 to a source module by name.** `foo_test` ↔ `foo`.
  An orphan unit test with no matching module fails.
- **No misplaced tests.** Nothing in the unit tree is secretly an integration
  test, or vice versa.

### Coverage integrity

- **Thresholds are declared once and applied to every language** (line / branch
  / function / statement). Opinionated default: **100%**, overridable.
- **Coverage is measured on the unit suite only.** Integration runs never count
  toward the number.
- **Changed lines must be unit-covered** (patch coverage), with a
  **non-regression floor** — coverage can't drop.
- **Test files are excluded from the coverage denominator.** You measure your
  source, not your tests.

### Test isolation

- **Unit tests must mock first-party collaborators.** Importing a real sibling
  module in a unit test fails, with an explicit `waiver: <reason>` escape hatch.
  *(Python: flake8 `MIS001` today; an ESLint equivalent is planned.)*
- **Integration tests must stub the outside world** — third-party packages,
  network, filesystem, clock, randomness — while using real first-party wiring.
  *(The hardest rule to enforce precisely; see [Roadmap](#roadmap).)*

### Packaging

- **Test files never ship** in the built artifact (wheel, sdist, npm `dist`).
  This is in scope precisely *because* colocated unit tests live alongside
  source in `src/`.

### Self-guard

- **The rules are CI-enforced, not advisory**, and the convention config itself
  is validated — so the boundary can't quietly rot.

### Not gates — nudges

Some good practices can't be enforced statically, so the tool **scaffolds and
reminds**, but never fails CI on them:

- Red-first TDD (write the failing test first).
- "Manually exercise new features" before calling them done.

These are kept separate from the hard gates on purpose. Conflating "we checked
this" with "we suggest this" is how trust in a gate erodes.

## What's deliberately out of scope

Keeping the label honest — `testing-conventions` is about *testing enforcement,
exclusively.* It does **not** do:

- **Release-docs gates** — changelog / docs / migration enforcement. A different
  concern (release, not testing).
- **Code style** — import conventions, formatting. That's your linter's job.
- **Unenforceable process** — the nudges above are scaffolded, not gated.

## How it works

`testing-conventions` is a **hybrid**: a cross-language CLI plus native in-test
lint plugins. That split isn't incidental — it's the only way to cover both
kinds of rule.

- **Structural CLI** — cross-language, run via `npx` / `uvx`. Implements the
  boundary, coverage-integrity, packaging, and self-guard rules. These are about
  *files, locations, and numbers*, so one tool can check them for any language.
- **Native lint plugins** — a `flake8` plugin (Python) and an ESLint rule (JS).
  The isolation and naming rules require AST analysis that has to run *inside*
  each ecosystem's linter, so they can't collapse into the cross-language CLI.

Everything reads from **one shared config** — the single source of truth for
what a unit test is and where tests live. Because all the rules key off that one
model, they live in **one monorepo** rather than scattered micro-repos.

## Configuration

A single, cross-language source of truth. Each `target` declares its language,
where unit code and tests live, the test-naming pattern, the integration
directory, the coverage command, thresholds, and any waivers.

```toml
[[target]]
language = "python"
unit_roots = ["src/mypackage"]
unit_test_glob = "**/*_test.py"
integration_dir = "tests"
coverage = { command = "pytest {unit_roots}", line = 100, branch = 100 }

[[target]]
language = "typescript"
unit_roots = ["src"]
unit_test_glob = "**/*.test.ts"
integration_dir = "tests"
coverage = { line = 100, branch = 100, function = 100, statement = 100 }
```

Native-language config files (`testconventions.py`, `testconventions.ts`) will
be supported alongside `test-conventions.toml`, for repos that prefer code over
a static file.

## Roadmap

The conventions are proven; packaging them into a standalone tool is in
progress. Open questions being worked through:

- **CLI runtime** — a single Rust/Go binary dual-published to npm + PyPI (the
  ruff / biome model) vs. a Python-first MVP shipped via `uvx`. Driven by which
  ecosystems the first consumers use.
- **Config format priority** — TOML-first vs. native-language-config-first.
- **"Stub the outside world"** — the integration-isolation rule. Likely a
  heuristic over known-external imports and un-fixtured I/O, or a declared
  allowlist. The single hardest rule; may land after the rest.
- **ESLint isolation/naming rule** — the JS counterpart to the existing flake8
  plugin.

## Design principles

Hard-won from building these gates the first time:

- **Prove every gate red-first.** A convention gate is worthless until you've
  watched it *fail on a real violation in CI.* Every rule ships with a red
  fixture (a violation it catches) and a clean fixture (that passes).
- **Separate hard gates from nudges**, in the docs and in the tool's output.
  Static and deterministic on one side; suggestions on the other. Don't blur
  them.
- **Dogfood.** A tool that demands isolated unit tests has isolated unit tests
  of its own.

## Prior art

The monorepo-with-coordinated-publishing shape — one repository, a small set of
coordinated packages — follows established tools like ruff, biome, and Babel.

## License

Released under the [MIT License](LICENSE).
