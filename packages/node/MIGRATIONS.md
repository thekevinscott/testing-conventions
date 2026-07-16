# Migrations

Upgrade notes for breaking changes. New entries go under `## Unreleased`.
On release, the section is renamed to `## v<OLD> → v<NEW>`.

Each entry has five sections, in order:

1. **Summary** — one paragraph: what changed and why.
2. **Required changes** — before/after for public API. "None" if purely additive.
3. **Deprecations removed** — anything previously warned about that's now gone.
4. **Behavior changes without code changes** — same API, different runtime behavior.
5. **Verification** — commands that confirm the upgrade worked, with expected output.

## Unreleased

### Summary

Retires the exact-match e2e freshness contract in favor of one branch-keyed decision per branch
(shipped through this package's bundled CLI; full notes in `packages/rust/MIGRATIONS.md`).
`e2e attest '<cmd>'` writes `e2e-attestations/<branch-slug>.json` and prunes receipts other
branches left behind; `e2e verify --base <ref>` passes a branch whose diff leaves the scoped
source untouched or carries a receipt, comparing no commit SHAs. A branch open across the upgrade
that changed scoped source runs `e2e attest '<cmd>'` once; the retired `e2e-attestation.json` is
collected by that same attest. `attest` must run on a checked-out branch. The new
`e2e slug [branch]` subcommand prints the standardized receipt slug, so scripts can locate a
branch's receipt at `e2e-attestations/$(npx testing-conventions e2e slug).json`.

Adds the TypeScript mutation engine adapter (#246, part of #239), organized by folder:
`src/mutation/mutation-cli.ts` exposes `mutationCLI` over one-function-per-file helpers alongside it,
and `src/mutation/main.ts` is the executable. It runs Stryker via its Node API and normalizes
`MutantResult[]` to the shared schema, selecting the bundled vitest runner. The Rust binary spawns it
(`dist/mutation/main.js`) for `unit mutation --language typescript`, and the launcher
(`src/bin/index.ts`) passes its path to the binary as a `--ts-mutation-adapter` argument (appended
only to a `unit mutation` invocation). Adds `@stryker-mutator/api` as a devDependency (engine result
types only). The `bin` entry now resolves to `dist/bin/index.js`; the package's `.` export is
unchanged.

Declares the TypeScript mutation engine (`@stryker-mutator/core`, `@stryker-mutator/vitest-runner`,
`^9.6.0`) as runtime dependencies, so installing testing-conventions brings them in and the adapter
resolves them from the package's own tree; the tool drives Stryker, and the consumer provides their own
test runner (`vitest`), which stays an optional peer (Stryker's vitest-runner peers on it). The one
install-time constraint: Stryker 9 sets the floor at **Node ≥20**.

Adds a `vitestConfig` export to the package root so consumers extend the shared
coverage floor instead of copy-pasting it. Purely additive: the `bin` entry and
its CLI behavior are unchanged, and `vitest` is added as an *optional* peer
dependency, so CLI-only consumers see no new install requirement.

Adds `--vitest-dir <path>` to the bundled mutation adapter (part of the package-root sandbox
fix; full notes in `packages/rust/MIGRATIONS.md`): the Rust core passes the scan path so
vitest's test discovery inside Stryker's sandbox stays scoped to the colocated unit suite.
Purely additive to an internal executable the binary spawns.

Sets Stryker's `inPlace: true` in the bundled mutation adapter (#460; full notes in
`packages/rust/MIGRATIONS.md`): Stryker applies each mutant to the package's real tree, backing
files up under `.stryker-tmp` and restoring them when the run ends, and reads the package's
`tsconfig.json` where it lies. Running in place keeps Stryker's sandbox ts-config preprocessor —
which imports `typescript` from `@stryker-mutator/core`'s own location, a package this package's
isolated install does not carry — out of the run. The adapter sets the option on every run; a
consumer's `{ "inPlace": true }` workaround config is now inert and can be deleted.

### Required changes

None to code. Install-time: the bundled Stryker 9 requires **Node ≥20** — a consumer on an older Node
must upgrade Node to install testing-conventions.

### Deprecations removed

None.

### Behavior changes without code changes

None.

### Verification

None.
