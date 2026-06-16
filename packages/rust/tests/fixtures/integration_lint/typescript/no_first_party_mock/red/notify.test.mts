import { describe, expect, it, vi } from 'vitest';

import { notify } from './notify-impl.mjs';

// VIOLATION via the `vi.doMock` form: `./mailer` is still a first-party module.
vi.doMock('./mailer', () => ({
  send: vi.fn(),
}));

describe('notify (integration)', () => {
  it('sends a notification', async () => {
    await notify('hello');
    expect(true).toBe(true);
  });
});
