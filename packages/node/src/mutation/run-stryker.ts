import { createRequire } from 'node:module';
import { Stryker } from '@stryker-mutator/core';
import type { PartialStrykerOptions } from '@stryker-mutator/api/core';

import { toNormalized, type NormalizedMutant } from './to-normalized.js';

/** Options for {@link runStryker}. */
export interface RunStrykerOptions {
  /**
   * Stryker `mutate` patterns to scope the run to (e.g. `<file>:<start>-<end>` ranges for a
   * diff-scoped gate). Omitted ⇒ Stryker's configured/default `mutate` set.
   */
  mutate?: string[];
  /**
   * Stryker `testFiles` patterns, relative to the project root: the suites that judge the
   * mutants. Omitted ⇒ the runner's own discovery.
   */
  testFiles?: string[];
}

// Stryker discovers plugins relative to the *project* under test, so a consumer project never
// finds our bundled `@stryker-mutator/vitest-runner`; passing the resolved path loads our copy.
const vitestRunnerPlugin = createRequire(import.meta.url).resolve('@stryker-mutator/vitest-runner');

/**
 * Run Stryker over the project in the current working directory via its Node API (#239) and
 * return the normalized results. Selects the **bundled** vitest runner explicitly by path (so
 * the unit-scoped runner is always used rather than Stryker's default `npm test` command
 * runner, #240, and resolves regardless of the project's location); results come from
 * `runMutationTest()` directly, so there is no report file. Because this runs inside the npm
 * package, `@stryker-mutator/core` resolves natively.
 */
export async function runStryker(options: RunStrykerOptions = {}): Promise<NormalizedMutant[]> {
  const cliOptions: PartialStrykerOptions = {
    testRunner: 'vitest',
    plugins: [vitestRunnerPlugin],
    // In place, so the run resolves through the project's own node_modules; a sandbox copy
    // would invoke Stryker's ts-config preprocessor, which needs a `typescript` this package's
    // production install does not carry.
    inPlace: true,
    reporters: [],
    ...(options.mutate ? { mutate: options.mutate } : {}),
    ...(options.testFiles === undefined ? {} : { testFiles: options.testFiles }),
  };
  const results = await new Stryker(cliOptions).runMutationTest();
  const projectRoot = process.cwd();
  return results
    .map((result) => toNormalized(result, projectRoot))
    .filter((mutant): mutant is NormalizedMutant => mutant !== null);
}
