import { describe, it, expect } from 'vitest';
import { VERSION, isPositive } from './index';

describe('VERSION', () => {
  it('reads the manifest one level above the scan path', () => {
    expect(VERSION).toBe('1.2.3');
  });
});

describe('isPositive', () => {
  it('pins the boundary on both sides', () => {
    expect(isPositive(5)).toBe(true);
    expect(isPositive(-5)).toBe(false);
    expect(isPositive(0)).toBe(false);
  });
});
