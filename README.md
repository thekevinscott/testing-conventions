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

## License

Released under the [MIT License](LICENSE).
