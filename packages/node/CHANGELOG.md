# Changelog

All notable changes to this package are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

### Fixed

- **The mutation adapter runs Stryker in place** (`inPlace: true`; #460, the gate-level behavior
  change is recorded in `packages/rust/CHANGELOG.md`). Stryker applies each mutant to the
  package's real tree — a backup lives under `.stryker-tmp` and the run restores every file when
  it ends — and reads the package's `tsconfig.json` where it lies. Running against the real tree
  keeps Stryker's sandbox ts-config preprocessor out of the run; that preprocessor rewrites
  sandbox copies by importing `typescript` from `@stryker-mutator/core`'s own location, a package
  the isolated production install (`npx -y testing-conventions`) does not carry, so on any
  package with a `tsconfig.json` the copy-sandbox run died at startup with
  `ERR_MODULE_NOT_FOUND`. In-place composes with the package-root rooting and `--vitest-dir`
  scoping: the run stays scan-path-scoped and the colocated unit suite judges the mutants. The
  adapter sets `inPlace` itself — it is the execution model, not a default a `stryker.conf.json`
  overrides.
- **The mutation adapter scopes vitest to the scan path within the package** (part of the
  package-root sandbox fix recorded in `packages/rust/CHANGELOG.md`). The adapter accepts a
  `--vitest-dir <path>` argument and passes it through as the vitest runner's `vitest.dir`, so
  when the Rust core roots Stryker's sandbox at the package root, the mutants are still judged by
  the scan path's colocated unit suite alone — the package's `tests/` tiers stay out of the run.

### Changed

- **E2E attestation is one branch-keyed decision per branch** (shipped through this package's
  bundled CLI; the full contract lives in `packages/rust/CHANGELOG.md`). `e2e attest '<cmd>'`
  writes `e2e-attestations/<branch-slug>.json` — parallel pull requests write distinct files and
  never merge-conflict — and the unrestricted command is the judgment the receipt records.
  `e2e verify --base <ref>` reads the branch's content diff: a branch that left the scoped source
  untouched owes nothing, one that changed it passes when its diff adds or updates a receipt, and
  later pushes, rebases, and squash merges never disturb a receipt. The new **`e2e slug [branch]`**
  subcommand prints the standardized receipt slug. **Breaking:** the single `e2e-attestation.json`
  is retired; `attest` collects a committed one automatically. See `MIGRATIONS.md`.

### Added

- **TypeScript mutation engine adapter** (#246, part of the #239 epic). Organized by folder:
  `src/mutation/mutation-cli.ts` exposes `mutationCLI` — the async orchestrator — over
  one-function-per-file helpers alongside it (`parse-args`, `run-stryker`, `to-normalized`,
  `normalize-status`), and `src/mutation/main.ts` is the executable that runs it. The adapter drives Stryker through its
  **Node API** (`new Stryker(opts).runMutationTest()`) and maps the structured `MutantResult[]` onto
  the normalized schema the Rust core gates on, selecting the bundled `@stryker-mutator/vitest-runner`
  by resolved path so the unit-scoped runner runs (#240) and reading results in-process (written to a
  `--out` file). The Rust binary spawns the adapter (`dist/mutation/main.js`) for `unit mutation
  --language typescript`; the launcher (`src/bin/index.ts`) passes its path to the binary as a
  `--ts-mutation-adapter` argument. Adds `@stryker-mutator/api` as a devDependency (the engine's
  result types). The `bin` entry now resolves to `dist/bin/index.js`; the package's `.` export is
  unchanged.
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
