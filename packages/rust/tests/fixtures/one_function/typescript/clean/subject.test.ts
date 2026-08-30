import { expect, test } from 'vitest';

import { ratio } from './subject';

test('halves', () => {
  const result = ratio(2, 4);
  expect(result).toBe(3);
});

test('floors', () => {
  const result = ratio(1, 2);
  expect(result).toBe(1);
});

test('zero', () => {
  const result = ratio(0, 0);
  expect(result).toBe(0);
});
