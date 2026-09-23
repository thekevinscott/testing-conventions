import { defineConfig, mergeConfig } from 'vitest/config';

import { vitestConfig } from './src/vitest-config';

// Extend `vitestConfig` rather than re-declaring the coverage floor, so this package is held
// to the exact standard it exports. The scripts glob is added here, not in `vitestConfig`
// itself: that export is shipped config a consumer extends, and this package's own build
// tooling under `scripts/` is not part of the standard those consumers are held to.
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
