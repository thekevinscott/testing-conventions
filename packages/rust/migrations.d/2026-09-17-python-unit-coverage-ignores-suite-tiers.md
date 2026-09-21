### Python `unit coverage` ignores the suite tiers under `tests/`

**Summary**

`run_coverage` ran `coverage run --source=. -m pytest .`, passing pytest the same root `--source`
scoped for measurement. When the scanned `source` coincides with the package root — a
package with no separate `src/`-style nesting — that root's `tests/integration/` and `tests/e2e/`
sat directly beneath it, so pytest collected and ran them as part of the unit suite: the unit lane
inherited the integration tier's environment preconditions, the tiers stopped being separable, and
a line covered only by an integration test could satisfy the unit-tier floor. The pytest invocation
now also passes `--ignore=tests`, so a `tests/` directory directly beneath the scanned path is
never collected — the same exclusion `integration-lint` and `colocated-test` already apply, and the
same outcome `cargo llvm-cov --lib` and vitest's own default include glob already give Rust and
TypeScript.

**Required changes**

_None._ The change is inside the Python coverage run; a `uses:` call or direct
`testing-conventions unit coverage --language python <source>` needs no edit.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A Python package whose scanned `source` is its package root, and which keeps its integration or
e2e suite under `tests/`, no longer runs that suite as part of `unit coverage`. A package whose
`source` nests below the package root (the documented layout, and this repo's own dogfood setup)
is unaffected — its `tests/` directory was never beneath the scanned path either before or after
this change. A package that relied on the integration suite's coverage to clear the unit floor now
measures the unit tier alone, and may need unit tests for lines the integration suite was covering
incidentally.

**Verification**

Run the gate on a Python package whose scanned path is its package root and holds a
`tests/integration/` directory beside its sources:

```console
$ testing-conventions unit coverage --language python .
```

The run's pytest output shows only colocated `*_test.py` files collected — no output from
`tests/integration/` or `tests/e2e/`.
