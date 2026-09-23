import { defineConfig, mergeConfig } from 'vitest/config';

import { vitestConfig } from './src/vitest-config';

// Extend `vitestConfig` rather than re-declaring the coverage floor, so this package is held
// to the exact standard it exports. The scripts glob stays out of that shipped export — see
// docs/internals/repo.md.
export default mergeConfig(
  vitestConfig,
  defineConfig({
    test: {
      include: ['scripts/**/*.test.ts'],
      coverage: {
        reporter: ['json', 'lcov'],
      },
    },
  }),
);
