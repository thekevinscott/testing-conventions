import { describe, expect, it } from 'vitest';

// A test file directly under `tests/` — outside every standard tier.
describe('loose', () => {
  it('passes', () => {
    expect(true).toBe(true);
  });
});
