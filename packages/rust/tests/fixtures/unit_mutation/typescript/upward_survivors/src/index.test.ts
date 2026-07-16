import { describe, it, expect } from 'vitest';
import { VERSION, isPositive } from './index';

describe('the suite runs the code but pins nothing', () => {
  it('runs VERSION', () => {
    expect(typeof VERSION).toBe('string');
  });

  it('runs isPositive', () => {
    expect(typeof isPositive(5)).toBe('boolean');
  });
});
