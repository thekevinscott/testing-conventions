import { defineConfig } from 'vitest/config';

// The e2e suite is deliberately separate from the unit `test` run
// (vite.config.ts, which only includes `src/**`) and is never run in CI: it
// drives the real, source-built CLI. Run it locally with `npm run test:e2e`,
// then record the run with `testing-conventions e2e attest 'npm run test:e2e'`;
// CI only verifies the committed attestation is current (#71).
export default defineConfig({
  test: {
    include: ['tests/e2e/**/*.test.ts'],
  },
});
