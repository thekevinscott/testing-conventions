import { describe, expect, it } from 'vitest';

import { normalizeStatus } from './normalize-status.js';

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
