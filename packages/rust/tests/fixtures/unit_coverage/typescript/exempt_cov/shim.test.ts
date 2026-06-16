import { expect, test } from 'vitest';

import * as shim from './shim';

test('importable', () => {
  expect(shim).toBeDefined();
});
