import { defineConfig, mergeConfig } from 'vitest/config';

import { vitestConfig } from './src/vitest-config';

// Extend `vitestConfig` rather than re-declaring the coverage floor, so this package is held
// to the exact standard it exports.
export default mergeConfig(
  vitestConfig,
  defineConfig({
    test: {
      coverage: {
        reporter: ['text', 'json', 'lcov'],
      },
    },
  }),
);
