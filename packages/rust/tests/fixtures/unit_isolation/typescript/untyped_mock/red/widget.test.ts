import { describe, expect, it, vi } from 'vitest';

import { makeWidget } from './widget';
import { format } from './formatter';
import { chunk } from 'lodash';

// Typed mock — anchored to the real module's type, so it can't drift. Fine.
vi.mock('./formatter', async () => {
  const actual = await vi.importActual<typeof import('./formatter')>('./formatter');
  return { ...actual, format: vi.fn() };
});

// UNTYPED mock — a factory with no `vi.importActual<…>` anchor to the real module.
// If `lodash`'s shape changes, this double won't catch it → untyped-mock violation.
vi.mock('lodash', () => ({ chunk: vi.fn() }));

describe('makeWidget', () => {
  it('builds a widget', () => {
    expect(makeWidget({ format, chunk })).toBeTruthy();
  });
});
