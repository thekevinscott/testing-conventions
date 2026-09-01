import { defineConfig } from 'vitest/config';

/** The shared vitest base config: the TypeScript coverage default the CLI enforces, for a
 * consumer to extend with `mergeConfig`. Keep the numbers in step with that default. */
export const vitestConfig = defineConfig({
  test: {
    include: ['src/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      include: ['src/**/*.ts'],
      exclude: ['src/**/*.d.ts'],
      thresholds: { lines: 100, branches: 100, functions: 100, statements: 100 },
    },
  },
});
