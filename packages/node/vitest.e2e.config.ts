import { defineConfig } from 'vitest/config';

// The e2e suite drives the real, source-built CLI, so it runs locally via `npm run test:e2e`
// and is recorded with `testing-conventions e2e attest`; CI verifies the attestation only.
export default defineConfig({
  test: {
    include: ['tests/e2e/**/*.test.ts'],
  },
});
