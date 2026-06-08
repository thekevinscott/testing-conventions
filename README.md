# testing-conventions

**Enforce testing conventions in libraries (Python, Typescript, and Rust)**

`testing-conventions` is an opinionated, config-driven standard that enforces
how tests are expected to function in a library.

1. What tests are (e.g., The line between unit and integration tests)
2. Where tests live
3. How tests are written
4. Coverage floors

## 1. What Tests Are
Tests exist to assert the behavior of code. They're there to provide confidence that the code does what it's supposed to do.

This library supports 4 kinds of tests:

1. Unit tests
2. Integration tests
3. E2E tests
4. Doc tests

Unit tests are cheap and plentiful but provide limited confidence. They're helpful in an agentic world though, particularly around refactoring.

Integration tests treat the system as a black box, with the caveat that expensive third party dependencies (databases, LLMs, system operations) are mocked.

E2E are identical to integration _without_ mocks. They're not meant to be run in CI, but they _are_ meant to be executable by LLMs to assert that the third party contracts are being honored.

Doc tests are for inline documentation.

## 2. Where Tests Live

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

#### Doc Tests
Doctests are runnable examples in docstrings, written in the interactive `>>>` style and run with `pytest --doctest-modules` (stdlib `doctest`). They keep documented examples honest. E.g.:

```python
def add(a: int, b: int) -> int:
    """
    >>> add(2, 2)
    4
    """
    return a + b
```

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

#### Doc Tests
Doctests are runnable examples in JSDoc `@example` comments (and Markdown), run as part of `vitest` via [`vite-plugin-doctest`](https://github.com/ssssota/doc-vitest). Mark the example fence with `@import.meta.vitest` and assert with `expect()`. E.g.:

````ts
/**
 * @example
 * ```ts @import.meta.vitest
 * expect(add(1, 2)).toBe(3);
 * ```
 */
export function add(a: number, b: number) {
  return a + b;
}
````

### Rust

#### Unit Tests
Unit tests are colocated with the code they test, in an inline `#[cfg(test)]` module in the same `.rs` file. This is the idiomatic Rust pattern, and it lets unit tests reach private items. E.g.:

```rust
// foo.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds() {
        assert_eq!(add(2, 2), 4);
    }
}
```

#### Integration Tests
Integration tests live in the top-level `tests/` directory (a sibling of `src/`). Cargo compiles each file there as its own crate, so they exercise only the crate's public API — no suffix needed, the location is the signal. Shared helper code goes in `tests/common/mod.rs`, which Cargo treats as a module rather than a test crate.

#### E2E Tests
E2E tests also live under `tests/`, driving the built binary (e.g. via `CARGO_BIN_EXE_<name>` or `assert_cmd`). Keep them in their own file(s), such as `tests/e2e.rs`, to separate them from the API-level integration suite.

#### Doc Tests
Doctests are runnable examples in `///` doc comments, compiled and run natively by `cargo test` (reported under "Doc-tests"). They exercise the public API and render into rustdoc. E.g.:

````rust
/// Adds two numbers.
///
/// ```
/// assert_eq!(mycrate::add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
````


## 3. How tests are written

### Python

In Python, use `pytest`.

#### Unit Tests
In unit tests, _everything_ except for the function under test should be mocked. Use `pytest

#### Integration Tests
In integration tests, third party dependencies should be mocked.

If the library exports a Python SDK, then mocking should be easy. If the library exposes a CLI, generally good practice is to primarily support a Python SDK that the CLI wraps; and for integration tests to _exclusively_ test the SDK.

### Typescript

In Typescript, use `vitest`.

#### Unit Tests
In unit tests, _everything_ except for the function under test should be mocked.

Mocks should be typed like so:

```typescript
vi.mock('rimraf', async () => {
  const actual = await vi.importActual("rimraf") as typeof rimraf;
  return {
    ...actual,
    rimraf: vi.fn(),
  };
});
```

#### Integration Tests
In integration tests, third party dependencies should be mocked.

If the library exports a Typescript SDK, then mocking should be easy. If the library exposes a CLI, generally good practice is to primarily support a Typescript SDK that the CLI wraps; and for integration tests to _exclusively_ test the SDK.

### Rust

In Rust, use `cargo test`.

#### 4. Coverage Floors

Aim for 100% branch coverage wherever possible.

--- Old LLM generated under this line ---

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
