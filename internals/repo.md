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

A second #206 follow-up: zero-config Rust coverage also routed `packages/python` into the rust matrix, because `detect.has_rust_crate` matched a bare `Cargo.toml`. `packages/python` carries a `Cargo.toml` but generates its Rust sources at wheel-build time, so a plain checkout has no `.rs` — and the rust coverage/mutation jobs then ran `cargo` over absent sources and failed (`can't find … src/main.rs`). This stayed latent until a PR touched `testing-conventions.yml` (the only `dogfood.yml` trigger that re-runs the `packages/python` reusable-workflow call). The fix: `has_rust_crate` now requires a `Cargo.toml` **and** at least one `.rs` source, so a manifest with nothing to measure is not treated as a crate. Like any detection change, it only reaches the self-test / dogfood once a release moves `@v0`.

## Rolling release: how `@v0` advances

`@v0` is a **moving major tag**: consumers pin `…/testing-conventions.yml@v0` and `…/actions/detect@v0`, and the tag is force-moved forward on each release so every consumer tracks `main`. We own all consumers and fix forward — this is rolling release, the opposite of a semver pin.

The tag is advanced by a dedicated workflow, `.github/workflows/move-major-tag.yml`, **not** inline in `release.yml`. It is **gated on a successful publish**: it triggers via `workflow_run` on the `Release` workflow completing and runs only when `conclusion == 'success'` (on `main`). That gate is the one place this repo departs from the generic "move the tag on every push to `main`" recipe, and it is non-negotiable:

The reusable workflow runs the **published** binary (`npx testing-conventions` → latest on npm), but the workflow *file* is frozen at `@v0`. If `@v0` advanced to a commit whose workflow invokes a subcommand the npm-latest binary doesn't expose yet (a rename/addition — the #55 class of break), every consumer running in the publish window would get new-workflow + old-binary → `unrecognized subcommand`. Publishing the binary is this repo's analog of committing a built `dist/`: ship the runtime first, then move the tag. `needs: release` (#92) did this inline; `move-major-tag.yml` does it as a named, single-responsibility workflow.

Two safety properties:

- **Concurrency** (`group: move-major-tag`, `cancel-in-progress: true`): the newest release wins; a stale in-flight move is cancelled.
- **Forward-only**: the tag moves to the released SHA only when that SHA is a descendant of the current `@v0` (otherwise it's a no-op), so out-of-order release runs can never rewind `@v0`. It also bootstraps `@v0` on first run.

The wiring is guarded in CI (`rolling-release-wired` in `testing-conventions-selftest.yml`): a regression that re-introduces an inline or un-gated tag move fails the self-test.
