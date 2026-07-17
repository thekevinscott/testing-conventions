import { it, expect } from 'vitest';

// The package-level integration tier. The unit coverage gate measures the scan path's
// colocated suite alone; if it ever runs this tier, the run fails loudly.
it('is never run by the unit coverage gate', () => {
  expect.unreachable('the unit coverage gate ran a package-level suite tier');
});
