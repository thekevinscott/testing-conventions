import { expect, test } from 'vitest';

import { classify } from './core';

test('positive', () => {
  expect(classify(1)).toBe('positive');
});

test('nonpositive', () => {
  expect(classify(-1)).toBe('nonpositive');
});
