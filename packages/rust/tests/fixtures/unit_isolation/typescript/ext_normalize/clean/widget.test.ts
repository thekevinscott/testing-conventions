// Clean fixture for #393: Vitest resolves `./formatter` and `./formatter.js` to the
// same module, so a mock and its import may spell the extension differently and still
// match. Standard nodenext ESM writes imports with `.js` and mocks bare; the inverse
// spelling (bare import, `.js` mock) resolves identically. Both collaborators are
// mocked, so nothing is flagged.
import { describe, expect, it, vi } from 'vitest';

import { makeWidget } from './widget'; // unit under test

import { format } from './formatter.js'; // import carries `.js`
vi.mock('./formatter'); // mock written bare — same module

import { log } from './logger'; // import written bare
vi.mock('./logger.js'); // mock carries `.js` — same module

describe('makeWidget', () => {
  it('builds a widget', () => {
    expect(makeWidget({ format, log })).toBeTruthy();
  });
});
