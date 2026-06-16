import { describe, expect, it, vi } from 'vitest';

import { makeWidget } from './widget'; // unit under test
import { format } from './formatter';
import { chunk } from 'lodash';

// Vitest 3 options-object mock — NOT a factory. `{ spy: true }` wraps the real
// module, so the double can't drift from the source; there is nothing to
// type-anchor. (See #111: this must not be flagged `untyped-mock`.)
vi.mock('./formatter', { spy: true });
vi.mock('lodash', { spy: true });

describe('makeWidget', () => {
  it('builds a widget', () => {
    expect(makeWidget({ format, chunk })).toBeTruthy();
  });
});
