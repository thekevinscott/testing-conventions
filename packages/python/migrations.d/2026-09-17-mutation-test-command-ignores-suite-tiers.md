### Python `unit mutation` ignores the suite tiers under `tests/`

**Summary**

`render_config` built a `test-command` of `python3 -m pytest -x -q -p no:cacheprovider`, run by
cosmic-ray from the package root with no path argument narrowing collection. When the scanned
module path coincides with the package root, that root's `tests/integration/` and `tests/e2e/` sat
directly beneath it and were collected into every per-mutant run, the same scoping gap
`unit coverage`'s pytest invocation had. `test-command` now also carries `--ignore=tests`, so a
`tests/` directory directly beneath the run root is never collected — the same exclusion this PR
gives Python's `unit coverage`, and the outcome TypeScript's mutation run already has via its
vitest test runner, whose default test discovery is scoped to `source` and never reaches a
sibling `tests/` tree.

**Required changes**

_None._ The change is inside the bundled cosmic-ray config; a `uses:` call or direct
`testing-conventions unit mutation --language python <source>` needs no edit.

**Deprecations removed**

_None._

**Behavior changes without code changes**

A Python package whose scanned module path is its package root, and which keeps its integration
or e2e suite under `tests/`, no longer runs that suite as part of the per-mutant baseline. A
package whose module path nests below the package root is unaffected — its `tests/` directory was
never beneath the run root either before or after this change. A mutant previously killed only by
an integration-tier test now survives, and needs a colocated unit test to kill it instead.

**Verification**

Run the gate on a Python package whose scanned path is its package root and holds a
`tests/integration/` directory beside its sources:

```console
$ testing-conventions unit mutation --language python .
```

The per-mutant suite run shows no collection from `tests/integration/` or `tests/e2e/`.
