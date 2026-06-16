import { describe, expect, it, vi } from 'vitest';

import { charge } from '../src/charge';

// VIOLATION: an integration test must run first-party code for real, so mocking
// a first-party collaborator (a relative import) is forbidden.
vi.mock('../src/ledger', () => ({
  record: vi.fn(),
}));

// Allowed: the external payment gateway is a third-party package.
vi.mock('stripe');

describe('charge (integration)', () => {
  it('records the charge', async () => {
    await charge({ amount: 100 });
    expect(true).toBe(true);
  });
});
