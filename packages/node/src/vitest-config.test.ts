import { describe, expect, it } from 'vitest';

import { vitestConfig } from './vitest-config';

// The base config is plain data — the same recommendation the CLI enforces,
// expressed on a second surface. Pin its shape so the exported floor can't
// silently drift from the tool's TypeScript coverage default.
describe('vitestConfig', () => {
  const test = vitestConfig.test;
  const coverage = test?.coverage;

  it('enforces the 100% coverage floor on every metric', () => {
    expect(coverage && 'thresholds' in coverage ? coverage.thresholds : undefined).toEqual({
      lines: 100,
      branches: 100,
      functions: 100,
      statements: 100,
    });
  });

  it('measures the v8 provider over src, excluding declaration files', () => {
    expect(coverage?.provider).toBe('v8');
    expect(coverage && 'include' in coverage ? coverage.include : undefined).toEqual([
      'src/**/*.ts',
    ]);
    expect(coverage && 'exclude' in coverage ? coverage.exclude : undefined).toEqual([
      'src/**/*.d.ts',
    ]);
  });

  it('scopes the unit run to colocated src tests', () => {
    expect(test?.include).toEqual(['src/**/*.test.ts']);
  });
});
