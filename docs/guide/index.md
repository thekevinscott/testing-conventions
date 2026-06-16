# Guides

Task-oriented recipes for putting `testing-conventions` to work. Each assumes the CLI is
installed — see [Getting Started](../getting-started).

## Enforce colocated unit tests

**The rule:** unit tests are colocated with the code they test and named after it, so the
unit/integration boundary is structural (by location, not a tag or marker) and an orphaned
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
non-zero. (Rust needs no separate check here: inline `#[cfg(test)]` modules make colocation
and 1:1 naming automatic.) Files that genuinely shouldn't be tested — re-export barrels, a
launcher shim — get an explicit, reason-required exemption in config; see
[Exempt a file](./exemptions).

## Check unit-test coverage

Coverage floors are enforced on the **unit suite only**, with branch coverage on and test
files excluded from the denominator. Put the floor in your config and run `unit coverage`:

```toml
# testing-conventions.toml
[python]
coverage = { branch = true, fail_under = 90 }
```

```sh
testing-conventions unit coverage --language python --config testing-conventions.toml src/
```

It runs the suite under `coverage.py`, compares the total to `fail_under`, and exits non-zero
on a shortfall — so CI fails on a coverage regression. (`python`, `coverage`, and `pytest`
must be installed; Python is the only language wired today.)

## Wire it into CI

`unit colocated-test`'s non-zero exit is all a CI step needs — a failing check fails the job,
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
```

Or skip the boilerplate with the [reusable workflow](./ci) — one job that runs every rule.

## See also

- [Reference](../reference/) — the full CLI surface and config schema.
