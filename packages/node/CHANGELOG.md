# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

- **TypeScript mutation engine adapter** (#246, part of the #239 epic). `src/stryker-adapter.ts` drives
  Stryker through its **Node API** (`new Stryker(opts).runMutationTest()`) and maps the structured
  `MutantResult[]` onto the normalized schema the Rust core gates on — selecting the bundled
  `@stryker-mutator/vitest-runner` so the unit-scoped runner is always used (never Stryker's default
  `npm test` command runner, #240), and reading results in-process (no CLI spawn, no report file). Not
  yet wired into the rule — the CLI switch-over is #247. Adds `@stryker-mutator/api` as a devDependency
  (the engine's result types). The `bin` entry and package exports are unchanged.
- **The TypeScript mutation engine now ships with the package.** `@stryker-mutator/core` and
  `@stryker-mutator/vitest-runner` (`^9.6.0`) are declared as runtime dependencies, so an
  `npm install` / `npx` of testing-conventions installs them, and `unit mutation --language typescript`
  resolves them from the project's `node_modules` — no separate engine install. The test runner
  (`vitest`) stays the consumer's optional peer, since it runs *their* suite and Stryker's runner plugin
  already peers on it. CLI-only consumers who don't run mutation simply carry the unused dependency.
  Stryker 9 requires **Node ≥20** (its own floor); a consumer on an older Node won't be able to install.
- `vitestConfig`: a shared vitest base config exported from the package root
  (`import { vitestConfig } from 'testing-conventions'`). Extend it with
  `mergeConfig` to hold a consumer's local `vitest --coverage` run to the same
  floor CI enforces (v8 provider, `src/**` coverage scope, `.d.ts` excluded,
  `100/100/100/100` thresholds). `vitest` is now an optional peer dependency
  (only needed when importing `vitestConfig`; CLI-only consumers are unaffected).
  ([#217](https://github.com/thekevinscott/testing-conventions/issues/217))

### Changed

### Deprecated

### Removed

### Fixed
