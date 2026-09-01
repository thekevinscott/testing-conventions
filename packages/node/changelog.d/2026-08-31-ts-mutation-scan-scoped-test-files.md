**Changed** **BREAKING** **The bundled TypeScript mutation adapter takes `--test-files` in place
of `--vitest-dir`** (#569). `--vitest-dir` set vitest's discovery root, which is also the base its
`include` patterns resolve against, so scoping the run to a subdirectory rewrote a root-relative
`include` (`src/**/*.test.ts`) into `src/src/**` and the run found no tests. `--test-files
<a,b,…>` passes Stryker `testFiles` patterns through instead, package-root-relative like
`--mutate` beside it: the patterns bound which test files judge the mutants, and vitest's root
stays where the config lies.

The adapter's argv is the contract between the CLI binary and the copy of the adapter shipped in
the same npm release; the two move together. See
[`../migrations.d/2026-08-31-ts-mutation-scan-scoped-test-files.md`](../migrations.d/2026-08-31-ts-mutation-scan-scoped-test-files.md).
