import { expect, test } from 'vitest';

import { grade } from './grade';

test('a', () => {
  expect(grade(95)).toBe('A');
});

test('f', () => {
  expect(grade(50)).toBe('F');
});
