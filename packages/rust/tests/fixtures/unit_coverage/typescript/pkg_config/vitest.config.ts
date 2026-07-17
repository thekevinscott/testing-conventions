import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    setupFiles: ['./vitest.setup.ts'],
    coverage: {
      thresholds: { lines: 100, autoUpdate: true },
    },
  },
});