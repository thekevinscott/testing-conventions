import { describe, it, expect } from 'vitest';
import { isPositive } from './index';

describe('isPositive', () => {
  it('pins the boundary on both sides', () => {
    expect(isPositive(5)).toBe(true);
    expect(isPositive(-5)).toBe(false);
    expect(isPositive(0)).toBe(false);
  });
});
