# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Added

- **TypeScript mutation engine adapter** (#246, part of the #239 epic). A Node adapter — one exported
  function per file under `src/` (`run-stryker`, `to-normalized`, `normalize-status`, `parse-args`,
  and the `mutation-cli` entry) — drives Stryker through its **Node API**
  (`new Stryker(opts).runMutationTest()`) and maps the structured `MutantResult[]` onto the normalized
  schema the Rust core gates on. It selects the bundled `@stryker-mutator/vitest-runner` by resolved
  path so the unit-scoped runner runs (#240), and reads results in-process, writing them to a `--out`
  file. The Rust binary spawns the adapter for `unit mutation --language typescript`; `bin.ts` passes
  its `dist/` path to the binary as a `--ts-mutation-adapter` argument. Adds `@stryker-mutator/api` as
  a devDependency (the engine's result types). The `bin` entry and package exports are unchanged.
- **The TypeScript mutation engine ships with the package.** `@stryker-mutator/core` and
  `@stryker-mutator/vitest-runner` (`^9.6.0`) are declared as runtime dependencies, so installing
  testing-conventions brings them in and the adapter resolves them from the package's own tree; the
  tool drives Stryker, and the consumer provides their own test runner (`vitest`), which stays an
  optional peer since it runs *their* suite and Stryker's runner plugin peers on it. Stryker 9 sets the
  floor at **Node ≥20**.
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
