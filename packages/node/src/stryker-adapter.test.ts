import { afterEach, describe, expect, it, vi } from 'vitest';
import type { MutantResult, MutantStatus } from '@stryker-mutator/api/core';

// Stryker is the adapter's one collaborator; mock it so `runStryker` can be driven without
// a real mutation run (the real run is exercised end-to-end once the rule is wired, #247).
// `vi.hoisted` exposes the spies to the hoisted `vi.mock` factory.
const { runMutationTest, ctorOptions } = vi.hoisted(() => ({
  runMutationTest: vi.fn<() => Promise<MutantResult[]>>(),
  ctorOptions: [] as unknown[],
}));
vi.mock('@stryker-mutator/core', () => ({
  Stryker: class {
    constructor(options: unknown) {
      ctorOptions.push(options);
    }
    runMutationTest() {
      return runMutationTest();
    }
  },
}));

import { normalizeStatus, runStryker, toNormalized } from './stryker-adapter';

/** A `MutantResult` carrying only the fields the adapter reads; the rest are irrelevant. */
function result(over: {
  status: MutantStatus;
  fileName?: string;
  line?: number;
  mutatorName?: string;
  replacement?: string;
}): MutantResult {
  return {
    fileName: over.fileName ?? 'src/index.ts',
    location: { start: { line: over.line ?? 1, column: 1 }, end: { line: over.line ?? 1, column: 9 } },
    mutatorName: over.mutatorName ?? 'ConditionalExpression',
    replacement: over.replacement,
    status: over.status,
  } as unknown as MutantResult;
}

describe('normalizeStatus', () => {
  it('maps each viable Stryker status to its normalized counterpart', () => {
    expect(normalizeStatus('Survived')).toBe('survived');
    expect(normalizeStatus('Killed')).toBe('killed');
    expect(normalizeStatus('NoCoverage')).toBe('no_coverage');
    expect(normalizeStatus('Timeout')).toBe('timeout');
    expect(normalizeStatus('CompileError')).toBe('compile_error');
    expect(normalizeStatus('RuntimeError')).toBe('runtime_error');
  });

  it('returns null for the non-outcomes the gate ignores', () => {
    expect(normalizeStatus('Ignored')).toBeNull();
    expect(normalizeStatus('Pending')).toBeNull();
  });
});

describe('toNormalized', () => {
  it('maps a result, folding in the replacement when present', () => {
    expect(
      toNormalized(result({ status: 'Survived', fileName: 'src/a.ts', line: 7, replacement: 'true' })),
    ).toEqual({ file: 'src/a.ts', line: 7, status: 'survived', mutator: 'ConditionalExpression', replacement: 'true' });
  });

  it('omits replacement when Stryker reports none', () => {
    const normalized = toNormalized(result({ status: 'NoCoverage', mutatorName: 'ArithmeticOperator' }));
    expect(normalized).toEqual({ file: 'src/index.ts', line: 1, status: 'no_coverage', mutator: 'ArithmeticOperator' });
    expect(normalized && 'replacement' in normalized).toBe(false);
  });

  it('drops a result whose status the gate ignores', () => {
    expect(toNormalized(result({ status: 'Ignored' }))).toBeNull();
  });
});

describe('runStryker', () => {
  afterEach(() => {
    runMutationTest.mockReset();
    ctorOptions.length = 0;
  });

  it('selects the vitest runner and normalizes results, dropping ignored mutants', async () => {
    runMutationTest.mockResolvedValue([
      result({ status: 'Survived', fileName: 'src/a.ts', line: 2, replacement: 'true' }),
      result({ status: 'Ignored', fileName: 'src/a.ts', line: 3 }),
      result({ status: 'Killed', fileName: 'src/a.ts', line: 9 }),
    ]);

    const survivors = await runStryker();

    // The runner is pinned to vitest (not Stryker's default command runner) and no file
    // reporter is requested.
    expect(ctorOptions[0]).toMatchObject({ testRunner: 'vitest', reporters: [] });
    expect(ctorOptions[0]).not.toHaveProperty('mutate');
    // Ignored is dropped; survived + killed map through.
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
