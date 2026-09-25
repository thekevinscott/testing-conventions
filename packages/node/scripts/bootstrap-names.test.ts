import { describe, expect, it } from 'vitest';

import { bootstrapNames } from './bootstrap-names.js';

describe('bootstrapNames', () => {
  it('splits a comma-separated list and trims each name', () => {
    expect(bootstrapNames('testing-conventions, @testing-conventions/darwin-arm64')).toEqual([
      'testing-conventions',
      '@testing-conventions/darwin-arm64',
    ]);
  });

  it('keeps a single name', () => {
    expect(bootstrapNames('testing-conventions')).toEqual(['testing-conventions']);
  });

  it('drops a segment that is empty or only whitespace', () => {
    expect(bootstrapNames('a,, ,b')).toEqual(['a', 'b']);
  });

  it('throws naming the input when it carries no name at all', () => {
    expect(() => bootstrapNames('')).toThrow('`packages` names nothing to bootstrap: ""');
    expect(() => bootstrapNames('   ')).toThrow('`packages` names nothing to bootstrap: "   "');
    expect(() => bootstrapNames(',,')).toThrow('`packages` names nothing to bootstrap: ",,"');
  });
});
