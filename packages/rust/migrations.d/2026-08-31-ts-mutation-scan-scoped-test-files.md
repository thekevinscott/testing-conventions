### TypeScript mutation scopes the suite by test-file patterns, not by vitest's discovery root

**Summary**

`unit mutation --language typescript <subdir>` previously passed the scan path as vitest's
discovery directory. Vitest resolves `include` patterns against that directory, so a package-root
`vitest.config.ts` carrying `include: ['src/**/*.test.ts']` re-resolved to `src/src/**` and
matched nothing: the gate failed with `No tests were executed` and exit 1. The run now scopes the
suite with Stryker's `testFiles` patterns rooted at the package root, leaving vitest's root where
the config lies.

**Required changes**

_None._

**Deprecations removed**

_None._

**Behavior changes without code changes**

A package whose vitest config writes `include` relative to the package root and is scanned below
that root now runs its colocated suite instead of erroring. Two consequences follow on the next
run of an unchanged tree:

- A gate that was hard-failing on `No tests were executed` now reports mutants. Survivors that
  were never reachable become visible, and an un-exempted one fails the check on its own terms.
- The mutant count in the passing line (`… (<n> mutant(s) tested)`) is non-zero where the run
  previously never got that far.

Which suites judge the mutants is unchanged: the scan path's colocated tests, never the package's
other tiers.

**Verification**

```
testing-conventions unit mutation --language typescript packages/<pkg>/src
```

against a package whose `vitest.config.ts` names a root-relative `include`. The run reports a
mutant count rather than `No tests were executed`.

The CLI runs on node 24 or newer — npm resolves a bare name to the newest release the running node
satisfies.
