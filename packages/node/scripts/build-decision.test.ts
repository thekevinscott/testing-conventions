import { describe, expect, it } from 'vitest';

import { shouldBuildShim } from './build-decision';

describe('shouldBuildShim', () => {
  it.each(['', 'main', 'noarch'])('builds the shim for %j', (target) => {
    expect(shouldBuildShim(target)).toBe(true);
  });

  it('exits early for a per-triple row', () => {
    expect(shouldBuildShim('x86_64-unknown-linux-gnu')).toBe(false);
  });
});
