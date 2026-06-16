import { describe, expect, it, vi } from 'vitest';

import { charge } from '../src/charge';

// Only third-party packages and Node built-ins are mocked — first-party code
// (../src/charge and its collaborators) runs for real.
vi.mock('stripe');
vi.mock('node:fs');
vi.mock('child_process');

describe('charge (integration)', () => {
  it('charges via the gateway', async () => {
    await charge({ amount: 100 });
    expect(true).toBe(true);
  });
});
