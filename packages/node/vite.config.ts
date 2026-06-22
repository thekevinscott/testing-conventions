import { defineConfig, mergeConfig } from 'vitest/config';

import { vitestConfig } from './src/vitest-config';

// Dogfood the shipped base config (#217): extend `vitestConfig` rather than
// re-declaring the coverage floor here, so this package is held to the exact
// standard it exports. Project-only overrides go in the second argument.
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
