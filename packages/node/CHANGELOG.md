# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

- **The TypeScript mutation engine now ships with the package.** `@stryker-mutator/core` and
  `@stryker-mutator/vitest-runner` are declared as runtime dependencies, so an `npm install` / `npx`
  of testing-conventions installs them, and `unit mutation --language typescript` resolves them from the
  project's `node_modules` — no separate engine install. The test runner (`vitest`) stays the
  consumer's optional peer, since it runs *their* suite and Stryker's runner plugin already peers on
  it. CLI-only consumers who don't run mutation simply carry the unused dependency.
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
