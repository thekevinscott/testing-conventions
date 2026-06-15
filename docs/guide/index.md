# Guides

Task-oriented recipes for putting `testing-conventions` to work. Each assumes the CLI is
installed — see [Getting Started](../getting-started).

## Enforce unit-test location & naming

**The rule:** unit tests are colocated with the code they test and named after it, so the
unit/integration boundary is structural (by location, not a tag or marker) and an orphaned
test can't hide. The expected twin varies by language:

| Language   | Source                              | Expected unit test     | Not a subject                          |
| ---------- | ----------------------------------- | ---------------------- | -------------------------------------- |
| Python     | `foo.py`                            | `foo_test.py`          | `*_test.py`, `__init__.py`             |
| TypeScript | `foo.ts` / `.tsx` / `.mts` / `.cts` | `foo.test.<ext>`       | `*.test.*`, `*.d.ts` / `.d.mts` / `.d.cts` |

Run it over your source directory:

```sh
testing-conventions unit-location src/                     # Python (default)
testing-conventions unit-location --lang typescript src/   # TypeScript
```

Every source file without its colocated test is printed to stderr and the command exits
non-zero. (Rust needs no separate check here: inline `#[cfg(test)]` modules make colocation
and 1:1 naming automatic.)

## Wire it into CI

`unit-location`'s non-zero exit is all a CI step needs — a failing check fails the job,
with the offending files named in the log:

```yaml
# .github/workflows/conventions.yml
steps:
  - uses: actions/checkout@v4
  - name: Unit-test location (Python)
    run: testing-conventions unit-location src/
  - name: Unit-test location (TypeScript)
    run: testing-conventions unit-location --lang typescript src/
```

## See also

- [Reference](../reference/) — the full CLI surface and config schema.
