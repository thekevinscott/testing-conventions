# Repo-wide conventions

Cross-cutting rules that apply across all language packages. Language-specific guidance lives in `python-supervision.md`, `typescript-supervision.md`, `rust-supervision.md`.

## CHANGELOG + MIGRATIONS

The `CHANGELOG.md` and `MIGRATIONS.md` *files* live at each package root. The philosophy below is global — every language package follows it.

Every PR that changes public API touches both files. Enforced in CI; a `skip-changelog:` trailer bypasses the check for genuinely internal refactors.

**`CHANGELOG.md`** — Keep a Changelog format. New entries land under `## Unreleased`, grouped `Added` / `Changed` / `Deprecated` / `Removed` / `Fixed`. Breaking changes carry a `**BREAKING**` prefix and link to their `MIGRATIONS.md` section. On release, `## Unreleased` is renamed to `## v<OLD> → v<NEW>` and a fresh `## Unreleased` opens.

**`MIGRATIONS.md`** — lives at the package root. New entries land under `## Unreleased`. Each entry has five sections, in order:

1. **Summary** — one paragraph: what changed and why.
2. **Required changes** — before/after for config, CLI flags, function/method arguments, action inputs. "None" if purely additive.
3. **Deprecations removed** — anything previously warned about that's now gone. "None" if nothing was removed.
4. **Behavior changes without code changes** — same API, different runtime behavior (tag format, exit codes, defaults).
5. **Verification** — commands the consumer runs to confirm the upgrade worked, with the expected output.

Public-API surface for the purpose of these files: every exported value/type, every CLI flag, every config key, every observable artifact (tag format, GitHub Release body shape). Internal refactors, test-only changes, and docs-only edits stay out.

## Self-test and the `@v0` path

The reusable workflow (`.github/workflows/testing-conventions.yml`) drives the **published** tool — its `detect` step pins `…/actions/detect@v0`, and each rule job runs `npx testing-conventions` (no version → latest on npm). The self-test (`testing-conventions-selftest.yml`) calls that reusable workflow. So a change to *detection* (which rules fan out) or *rule behavior* does **not** take effect in the self-test — or for any consumer — until a release **moves `@v0`** to the new commit and publishes the package.

The trap: a change can stay green in its own PR's self-test (still running the old `@v0` path) yet break the self-test on the **next release**, when `@v0` advances. So any change that alters which rules a fixture is fanned over must leave every self-test fixture passing under the *new* path, not just the merged one. Concretely, a fixture driven through the reusable workflow (`uses:`) must pass **every** rule it could be fanned over — not only the rule it was added for.

Worked example (#206): making Rust coverage zero-config routed every detected Rust crate into the coverage matrix. The lint-only `integration-rust/clean` fixture then had to become coverage-clean too — its integration test runs first-party code for real (and so compiles and is fully line-covered under `cargo llvm-cov`) rather than carrying a `#[double]` that only ever parsed for the lint. Verify a fixture by hand with the published-equivalent command, e.g. `testing-conventions unit coverage --language rust .github/selftest/integration-rust/clean`, since the PR's own CI won't exercise the post-release path.
