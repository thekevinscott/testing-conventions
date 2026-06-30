import { describe, expect, it } from 'vitest';
import type { MutantResult, MutantStatus } from '@stryker-mutator/api/core';

import { toNormalized } from './to-normalized.js';

/** A `MutantResult` carrying only the fields the adapter reads; the rest are irrelevant. */
function result(over: {
  status: MutantStatus;
  fileName?: string;
  line?: number;
  mutatorName?: string;
  replacement?: string;
}): MutantResult {
  return {
    fileName: over.fileName ?? '/proj/src/index.ts',
    location: { start: { line: over.line ?? 1, column: 1 }, end: { line: over.line ?? 1, column: 9 } },
    mutatorName: over.mutatorName ?? 'ConditionalExpression',
    replacement: over.replacement,
    status: over.status,
  } as unknown as MutantResult;
}

describe('toNormalized', () => {
  it('relativizes the absolute fileName to the project root and folds in the replacement', () => {
    expect(
      toNormalized(result({ status: 'Survived', fileName: '/proj/src/a.ts', line: 7, replacement: 'true' }), '/proj'),
    ).toEqual({ file: 'src/a.ts', line: 7, status: 'survived', mutator: 'ConditionalExpression', replacement: 'true' });
  });

  it('leaves an already-relative fileName as-is and omits an absent replacement', () => {
    const normalized = toNormalized(result({ status: 'NoCoverage', fileName: 'src/b.ts', mutatorName: 'ArithmeticOperator' }), '/proj');
    expect(normalized).toEqual({ file: 'src/b.ts', line: 1, status: 'no_coverage', mutator: 'ArithmeticOperator' });
    expect(normalized && 'replacement' in normalized).toBe(false);
  });

  it('drops a result whose status the gate ignores', () => {
    expect(toNormalized(result({ status: 'Ignored' }), '/proj')).toBeNull();
  });
});
