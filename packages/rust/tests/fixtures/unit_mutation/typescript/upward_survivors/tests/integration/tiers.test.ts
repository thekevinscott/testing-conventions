import { it, expect } from 'vitest';

// The package-level integration tier. The unit mutation gate judges mutants by the scan
// path's colocated suite alone; if it ever runs this tier, the initial (dry) run fails loudly.
it('is never run by the unit mutation gate', () => {
  expect.unreachable('the unit mutation gate ran a package-level suite tier');
});
