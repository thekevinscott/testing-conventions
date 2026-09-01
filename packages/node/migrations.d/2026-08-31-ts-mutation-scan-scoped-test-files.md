### The mutation adapter's `--vitest-dir` becomes `--test-files`

**Summary**

The bundled TypeScript mutation adapter scoped the judging suite with `--vitest-dir <path>`,
which Stryker forwards as vitest's discovery directory — the base vitest resolves `include`
against. A root-relative `include` therefore re-resolved and the run found no tests. The flag is
replaced by `--test-files <a,b,…>`, a comma-separated list of Stryker `testFiles` patterns
addressed from the package root, and `runStryker` takes `testFiles: string[]` where it took
`vitestDir: string`.

**Required changes**

_None_ for anyone running the CLI: the binary supplies this argv, and the binary and the adapter
ship in the same npm release. A tree invoking `dist/mutation/main.js` directly replaces
`--vitest-dir src` with `--test-files 'src/**'`.

**Deprecations removed**

`--vitest-dir` and the `vitestDir` option on `runStryker` are gone; the adapter no longer sets
Stryker's `vitest.dir`.

**Behavior changes without code changes**

A mutation run over a subdirectory of a package whose vitest config writes `include` relative to
the package root now runs that package's colocated suite instead of failing with `No tests were
executed`.

**Verification**

```
node dist/mutation/main.js --test-files 'src/**' --mutate 'src/index.ts:1-40'
```

emits the normalized-mutant JSON for the scan path, judged by the test files under it.
