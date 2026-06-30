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
}

// The bundled vitest runner plugin's absolute path. Stryker discovers plugins relative to the
// *project* under test, not to where it was imported from — so when the rule runs the engine
// over a consumer project, our bundled `@stryker-mutator/vitest-runner` isn't on that search
// path. Passing its resolved path as an explicit plugin makes Stryker load our copy regardless
// of the project's location (#246). `vitest` itself stays the consumer's (the runner's peer).
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
    // Results come from runMutationTest()'s return value, so no file/stdout reporter is needed.
    reporters: [],
    ...(options.mutate ? { mutate: options.mutate } : {}),
  };
  const results = await new Stryker(cliOptions).runMutationTest();
  const projectRoot = process.cwd();
  return results
    .map((result) => toNormalized(result, projectRoot))
    .filter((mutant): mutant is NormalizedMutant => mutant !== null);
}
