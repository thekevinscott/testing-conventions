# testing-conventions

**Enforce testing conventions in libraries (Python, Typescript, and Rust)**

`testing-conventions` is an opinionated, config-driven standard that enforces
how tests are expected to function in a library. It promotes the following:

- Where tests live
- How tests are written
- The line between unit and integration tests
- Coverage floors

It works with Python, Typescript, and Rust.

## Where Tests Live

### Python

#### Unit Tests
Unit tests should be colocated with Python src code and appended with `_test.py`. E.g.:

```
- foo.py
- foo_test.py
```

#### Integration Tests
Integration tests should live in tests/integration and end in `_test.py`. Non-test helper code can live alongside the integration tests, but omit the suffix.

#### E2E Tests
E2E tests should live in tests/e2e and end in `_test.py`. Non-test helper code can live alongside the integration tests, but omit the suffix.

### Typescript

#### Unit Tests
Unit tests should be colocated with Typescript src code and appended with `.test.ts`. E.g.:

```
- foo-bar.ts
- foo-bar.test.ts
```

#### Integration Tests
Integration tests should live in tests/integration and end in `.test.ts`. Non-test helper code can live alongside the integration tests, but omit the suffix.

#### E2E Tests
E2E tests should live in tests/e2e and end in `test.ts`. Non-test helper code can live alongside the integration tests, but omit the suffix.

### Rust

#### Unit Tests
Unit tests should be colocated with Rust src code and appended with `_test.rs`. E.g.:

```
- foo.rs
- foo_test.rs
```

#### Integration Tests
Integration tests should live in tests/integration and end in `_test.rs`. Non-test helper code can live alongside the integration tests, but omit the suffix.

#### E2E Tests
E2E tests should live in tests/e2e and end in `_test.rs`. Non-test helper code can live alongside the integration tests, but omit the suffix.


## How tests are written

### Python

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
