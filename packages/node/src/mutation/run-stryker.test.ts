import { afterEach, describe, expect, it, vi } from 'vitest';
import type { MutantResult, MutantStatus } from '@stryker-mutator/api/core';

// Stryker is the one collaborator; mock it so `runStryker` runs without a real mutation run
// (the real run is exercised end-to-end by the rule's integration/e2e suites). `vi.hoisted`
// exposes the spies to the hoisted `vi.mock` factory.
const { runMutationTest, ctorOptions } = vi.hoisted(() => ({
  runMutationTest: vi.fn<() => Promise<MutantResult[]>>(),
  ctorOptions: [] as Array<Record<string, unknown>>,
}));
vi.mock('@stryker-mutator/core', () => ({
  Stryker: class {
    constructor(options: Record<string, unknown>) {
      ctorOptions.push(options);
    }
    runMutationTest() {
      return runMutationTest();
    }
  },
}));

import { runStryker } from './run-stryker.js';

/** A `MutantResult` with an absolute `fileName` (as Stryker reports), under the cwd. */
function result(status: MutantStatus, over: { file?: string; line?: number; replacement?: string } = {}): MutantResult {
  return {
    fileName: `${process.cwd()}/${over.file ?? 'src/a.ts'}`,
    location: { start: { line: over.line ?? 1, column: 1 }, end: { line: over.line ?? 1, column: 9 } },
    mutatorName: 'ConditionalExpression',
    replacement: over.replacement,
    status,
  } as unknown as MutantResult;
}

describe('runStryker', () => {
  afterEach(() => {
    runMutationTest.mockReset();
    ctorOptions.length = 0;
  });

  it('selects the vitest runner + its bundled plugin, relativizes paths, and drops ignored mutants', async () => {
    runMutationTest.mockResolvedValue([
      result('Survived', { file: 'src/a.ts', line: 2, replacement: 'true' }),
      result('Ignored', { file: 'src/a.ts', line: 3 }),
      result('Killed', { file: 'src/a.ts', line: 9 }),
    ]);

    const survivors = await runStryker();

    expect(ctorOptions[0]).toMatchObject({ testRunner: 'vitest', reporters: [] });
    // The bundled vitest-runner is passed by resolved path so Stryker loads our copy.
    expect(ctorOptions[0].plugins).toEqual([expect.stringContaining('vitest-runner')]);
    expect(ctorOptions[0]).not.toHaveProperty('mutate');
    expect(survivors).toEqual([
      { file: 'src/a.ts', line: 2, status: 'survived', mutator: 'ConditionalExpression', replacement: 'true' },
      { file: 'src/a.ts', line: 9, status: 'killed', mutator: 'ConditionalExpression' },
    ]);
  });

  it('passes through mutate ranges when given', async () => {
    runMutationTest.mockResolvedValue([]);

    await runStryker({ mutate: ['src/a.ts:2-4'] });

    expect(ctorOptions[0]).toMatchObject({ testRunner: 'vitest', mutate: ['src/a.ts:2-4'] });
  });
});
