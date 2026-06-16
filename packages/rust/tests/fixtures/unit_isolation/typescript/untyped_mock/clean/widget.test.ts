import { describe, expect, it, vi } from 'vitest';

import { makeWidget } from './widget';
import { format } from './formatter';
import { chunk } from 'lodash';

// Typed factory mock — `vi.importActual<typeof import(...)>` anchors it to the source.
vi.mock('./formatter', async () => {
  const actual = await vi.importActual<typeof import('./formatter')>('./formatter');
  return { ...actual, format: vi.fn() };
});

// Bare auto-mock — vitest derives the double from the real module's types, so it
// can't drift. No factory, nothing to type.
vi.mock('lodash');

describe('makeWidget', () => {
  it('builds a widget', () => {
    expect(makeWidget({ format, chunk })).toBeTruthy();
  });
});
