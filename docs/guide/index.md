# Guides

Task-oriented recipes for `testing-conventions`. Each assumes the CLI is installed (see
[Getting Started](../getting-started)).

## Enforce colocated unit tests

**The rule:** unit tests are colocated with the code they test and named after it. This makes
the unit/integration boundary structural (by location, not a tag or marker), and an orphaned
test can't hide. The expected twin varies by language:

| Language   | Source                              | Expected unit test     | Not a subject                          |
| ---------- | ----------------------------------- | ---------------------- | -------------------------------------- |
| Python     | `foo.py`                            | `foo_test.py`          | `*_test.py`, `__init__.py`             |
| TypeScript | `foo.ts` / `.tsx` / `.mts` / `.cts` | `foo.test.<ext>`       | `*.test.*`, `*.d.ts` / `.d.mts` / `.d.cts` |

Run it over your source directory:

```sh
testing-conventions unit colocated-test --language python src/       # Python
testing-conventions unit colocated-test --language typescript src/   # TypeScript
```

Every source file without its colocated test is printed to stderr and the command exits
non-zero. (Rust needs no separate check: inline `#[cfg(test)]` modules make colocation and 1:1
naming automatic.) Files that genuinely shouldn't be tested, such as re-export barrels or a
launcher shim, get an explicit, reason-required exemption in config; see
[Exempt a file](./exemptions).

## Check unit-test coverage

Coverage floors are enforced on the **unit suite only**, with test files excluded from the
denominator. Put the floors in your config and run `unit coverage`. Skip the config and `unit
coverage` uses the language's default floor (Python `fail_under = 85` with branch on; TypeScript
`lines`/`functions`/`statements` 80, `branches` 75). Rust is the exception: it has no default
floor yet, so give it an explicit `[rust].coverage` table.

**Python:** one total floor, branch coverage on, measured by `coverage.py`:

```toml
# testing-conventions.toml
[python]
coverage = { branch = true, fail_under = 90 }
```

```sh
testing-conventions unit coverage --language python --config testing-conventions.toml src/
```

It runs the suite under `coverage.py`, compares the total to `fail_under`, and exits non-zero
on a shortfall. (`python`, `coverage`, and `pytest` must be installed.)

**TypeScript:** four independent floors, measured by `vitest` v8 coverage:

```toml
# testing-conventions.toml
[typescript]
coverage = { lines = 90, branches = 80, functions = 90, statements = 90 }
```

```sh
testing-conventions unit coverage --language typescript --config testing-conventions.toml src/
```

It runs the suite under `vitest` (via `npx`, so `vitest` and `@vitest/coverage-v8` must be
installed), excludes `*.test.*` and declaration files, and exits non-zero naming any of the four
metrics below its floor, so CI fails when coverage drops below the floor. Measuring all four matters: line
coverage can read 100% while branches lag, when every line of a function runs but its `else` is
never taken.

**Rust** — two floors (regions and lines), measured by `cargo llvm-cov`:

```toml
# testing-conventions.toml
[rust]
coverage = { regions = 90, lines = 90 }
```

```sh
testing-conventions unit coverage --language rust --config testing-conventions.toml .
```

It runs `cargo llvm-cov` over the crate, compares the regions and lines totals to their floors, and
exits non-zero naming either below it (`cargo-llvm-cov` must be installed). Branch coverage is still
experimental, so it isn't enforced. Note a stable-toolchain caveat: inline `#[cfg(test)]` test code
can't be excluded from the denominator by filename (`#[coverage(off)]` is nightly), so it's measured
alongside the source — a `coverage` exemption still drops whole files via `--ignore-filename-regex`.

## Keep integration tests honest

An integration test runs first-party code for real and mocks only the outside world. A
`vi.mock()` of a first-party (relative) module breaks that, and the `no-first-party-mock` lint
catches it:

```sh
testing-conventions integration lint --language typescript test/integration/
```

Any `vi.mock()` / `vi.doMock()` of a `./`-relative module is printed and the command exits
non-zero; third-party packages and Node built-ins stay mockable. See [Isolate
tests](./isolation) for the full first-party/external rule and the unit-suite counterpart.

## Wire it into CI

`unit colocated-test`'s non-zero exit is all a CI step needs: a failing check fails the job,
with the offending files named in the log:

```yaml
# .github/workflows/conventions.yml
steps:
  - uses: actions/checkout@v4
  - name: Colocated test (Python)
    run: testing-conventions unit colocated-test --language python src/
  - name: Colocated test (TypeScript)
    run: testing-conventions unit colocated-test --language typescript src/
  - name: Unit-test coverage (Python)
    run: testing-conventions unit coverage --language python --config testing-conventions.toml src/
  - name: Unit-test coverage (TypeScript)
    run: testing-conventions unit coverage --language typescript --config testing-conventions.toml src/
```

Or skip the boilerplate with the [reusable workflow](./ci): one job that runs every rule.

## See also

- [Reference](../reference/): the full CLI surface and config schema.
