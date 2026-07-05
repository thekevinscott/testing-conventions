import { expect, test } from 'vitest';

import { classify } from './widget';

test('positive', () => {
  expect(classify(1)).toBe('positive');
});

test('negative', () => {
  expect(classify(-1)).toBe('negative');
});

test('zero', () => {
  expect(classify(0)).toBe('zero');
});
