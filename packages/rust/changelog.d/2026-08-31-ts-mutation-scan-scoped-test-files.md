**Fixed** **`unit mutation --language typescript` over a subdirectory runs the tests it should**
(#569). The run scoped vitest by handing the adapter the scan path as vitest's discovery
directory, which vitest also uses as the base its `include` patterns resolve against. A package
whose `vitest.config.ts` carries a root-relative `include` — `src/**/*.test.ts`, the shape this
tool's own `vitestConfig` export ships — therefore re-resolved it to `src/src/**`, matched no
test file, and the run died with `No tests were executed` and exit 1, naming nothing that pointed
at the rewrite.

The run now narrows the suite through Stryker's `testFiles` patterns instead, addressed from the
package root like the `mutate` patterns beside them. The scan path bounds **which** test files
judge the mutants; vitest stays rooted at the package root, so a root-relative `include` resolves
exactly as it does in your own runs, and the package's other suite tiers (`tests/`) stay out of
the run as before. See
[`../migrations.d/2026-08-31-ts-mutation-scan-scoped-test-files.md`](../migrations.d/2026-08-31-ts-mutation-scan-scoped-test-files.md).
