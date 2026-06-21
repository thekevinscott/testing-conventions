import { describe, it, expect } from 'vitest';
import { add, isPositive } from './index';

describe('add', () => {
  it('pins exact sums', () => {
    expect(add(2, 3)).toBe(5);
    expect(add(-1, 1)).toBe(0);
  });
});

describe('isPositive', () => {
  it('pins the boundary on both sides', () => {
    expect(isPositive(5)).toBe(true);
    expect(isPositive(-5)).toBe(false);
    expect(isPositive(0)).toBe(false);
  });
});
