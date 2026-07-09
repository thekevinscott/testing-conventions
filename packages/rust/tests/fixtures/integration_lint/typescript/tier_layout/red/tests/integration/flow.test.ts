import { describe, expect, it, vi } from 'vitest';

import { greet } from '../../src/widget';

// VIOLATION: an integration test must run first-party code for real, so mocking
// a first-party collaborator (a relative import) is forbidden — and the lint must
// find this suite even though the call's `path` is the sibling `src/` directory.
vi.mock('../../src/widget', () => ({
  greet: vi.fn(),
}));

describe('flow (integration)', () => {
  it('greets', () => {
    expect(greet('Ada')).toBeDefined();
  });
});
