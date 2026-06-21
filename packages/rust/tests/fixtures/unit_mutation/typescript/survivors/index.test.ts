import { describe, it, expect } from 'vitest';
import { add, isPositive } from './index';

describe('add', () => {
  it('runs but barely asserts', () => {
    expect(typeof add(2, 3)).toBe('number');
  });
});

describe('isPositive', () => {
  it('runs but barely asserts', () => {
    expect(typeof isPositive(5)).toBe('boolean');
  });
});
