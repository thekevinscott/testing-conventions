import { defineConfig } from 'vitest/config';

// The shared vitest base config (#217). Consumers extend it instead of
// copy-pasting our coverage floor (and drifting from it):
//
//   import { defineConfig, mergeConfig } from 'vitest/config';
//   import { vitestConfig } from 'testing-conventions';
//   export default mergeConfig(vitestConfig, defineConfig({ /* overrides */ }));
//
// It carries the same TypeScript coverage default the CLI enforces
// (100/100/100/100, branch on, v8 over `src`, declaration files excluded), so a
// local `vitest --coverage` run surfaces a shortfall before CI does. The numbers
// here are the one recommendation expressed on a second surface — keep them in
// step with the tool's TypeScript coverage default.
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
