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

Adds the TypeScript mutation engine adapter (#246, part of #239): `src/stryker-adapter.ts` runs Stryker
via its Node API and normalizes `MutantResult[]` to the shared schema, selecting the bundled vitest
runner. Additive and not yet wired into the rule (the CLI switch-over is #247); the `bin` entry and
package exports are unchanged. Adds `@stryker-mutator/api` as a devDependency (engine result types only).

Declares the TypeScript mutation engine (`@stryker-mutator/core`,
`@stryker-mutator/vitest-runner`, `^9.6.0`) as runtime dependencies, so installing testing-conventions
installs them automatically — `unit mutation --language typescript` resolves them from the project's
`node_modules` instead of requiring a separate install. Additive for the CLI: the `bin` entry and its
behavior are unchanged, and the test runner (`vitest`) stays the consumer's optional peer (Stryker's
vitest-runner peers on it). A CLI-only consumer who never runs mutation just carries the unused dep.
The one install-time constraint: Stryker 9 requires **Node ≥20**, so the package no longer installs on
older Node.

Adds a `vitestConfig` export to the package root so consumers extend the shared
coverage floor instead of copy-pasting it. Purely additive: the `bin` entry and
its CLI behavior are unchanged, and `vitest` is added as an *optional* peer
dependency, so CLI-only consumers see no new install requirement.

### Required changes

None to code. Install-time: the bundled Stryker 9 requires **Node ≥20** — a consumer on an older Node
must upgrade Node to install testing-conventions.

### Deprecations removed

None.

### Behavior changes without code changes

None.

### Verification

None.
