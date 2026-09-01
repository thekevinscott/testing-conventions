import { defineConfig } from 'vitest/config';

// The shape the tool's own `vitestConfig` export ships: `include` is written relative to the
// package root the config lives at.
export default defineConfig({
  test: {
    include: ['src/**/*.test.ts'],
  },
});
