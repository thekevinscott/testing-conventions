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

## Monorepo package-root derivation

`detect.py` derives six outputs a suite-executing job needs to install, build, run, and
configure at the right directory (#277, #289): `package_root`, `ts_package_manager`, `python_env`,
`provision_rust`, `config`, `build_command`. A `working_directory` input was considered and
rejected — it would
add a second, consumer-facing coordinate system against the documented rule that `path` is the
only scoping mechanism (docs/monorepo.md). Everything else is derived from `path` and the
package's own manifest instead.

- **`package_root`** (`derive_package_root`): the nearest directory at-or-above the scan root
  (`path`), down to the checkout root inclusive, holding a `package.json` / `pyproject.toml` /
  `Cargo.toml`; the checkout root (`.`) when none is found. A single-package repo has no manifest
  above the scan root other than possibly at the checkout root, so it always derives `.` —
  every current consumer is untouched.
- **`ts_package_manager`** (`ts_package_manager`): `package.json`'s `packageManager` field name,
  else `pnpm`/`npm` by lockfile presence, else `pnpm` (today's hardcoded default).
- **`python_env`** (`python_env`): `uv` when `package_root`'s `pyproject.toml` parses with a
  `[project]` table (an installable project with its own dependencies), else `pip` — no
  `pyproject.toml`, one with only tool config, or one that fails to parse. detect never crashes
  on a malformed manifest; it degrades to `pip`.
- **`provision_rust`** (`provision_rust`): `true` when `package_root`'s own manifest declares a
  Rust-compiling build — a `Cargo.toml` sits there, `pyproject.toml`'s
  `build-system.build-backend` is a maturin backend, or `package.json` declares a `napi` key or
  an `@napi-rs/cli` devDependency. `rust_toolchain` remains as a manual override for a build no
  manifest field expresses.
- **`config`** (`derive_config`): the `config` input verbatim when the caller named anything
  other than the default (`testing-conventions.toml`); otherwise a `testing-conventions.toml`
  at `package_root` when one exists there, else the default itself — today's repo-root behavior,
  unchanged when `package_root` is `.`. Every suite/lint job's `CONFIG` env reads this output
  instead of `inputs.config` directly, so a per-package call's own config file is discovered,
  never named.
- **`build_command`** (`derive_build_command`, #289): the `[python].build_command` shell command
  read from the discovered `config` file — the one escape hatch a PEP 517 Python backend can't
  express (npm's `prepare`/`postinstall` and Cargo's `build.rs` cover TypeScript and Rust). This is
  the only detect function that opens and parses a `testing-conventions.toml`'s *contents*, not just
  resolves its path; `''` when the file is absent, unparseable, or declares no build command. The
  suite-executing jobs run `needs.detect.outputs.build_command` before the suite, replacing the
  removed `build_command` *workflow input*.

These are the primitive the four gate fixes (#278–#281) consume; deriving them is out of scope
for what those jobs *do* with them (installing, building, discovering `dist/`, discovering
`e2e-attestation.json`) — see each issue for its own gate-specific wiring.

The `.github/selftest/monorepo/` fixture (no manifest or lockfile at its own root, mirroring a
real per-package-lockfile monorepo) exercises the derivation end to end via the local
`./.github/actions/detect` action — the same pattern `detect-routes-python` in
`testing-conventions-selftest.yml` already uses, so it isn't blocked by the `@v0` lag described
below.

## Self-test and the `@v0` path

The reusable workflow (`.github/workflows/testing-conventions.yml`) drives the **published** tool — its `detect` step pins `…/actions/detect@v0`, and each rule job runs `npx testing-conventions` (no version → latest on npm). The self-test (`testing-conventions-selftest.yml`) calls that reusable workflow. So a change to *detection* (which rules fan out) or *rule behavior* does **not** take effect in the self-test — or for any consumer — until a release **moves `@v0`** to the new commit and publishes the package.

The trap: a change can stay green in its own PR's self-test (still running the old `@v0` path) yet break the self-test on the **next release**, when `@v0` advances. So any change that alters which rules a fixture is fanned over must leave every self-test fixture passing under the *new* path, not just the merged one. Concretely, a fixture driven through the reusable workflow (`uses:`) must pass **every** rule it could be fanned over — not only the rule it was added for.

Worked example (#206): making Rust coverage zero-config routed every detected Rust crate into the coverage matrix. The lint-only `integration-rust/clean` fixture then had to become coverage-clean too — its integration test runs first-party code for real (and so compiles under `cargo llvm-cov`) rather than carrying a `#[double]` that only ever parsed for the lint. A second round (#265): scoping the Rust coverage arm to the unit suite (`--lib`) took the integration tier out of the number, so the fixture also carries an inline `#[cfg(test)]` test that covers `compute` — the unit suite clears the floor on its own. Verify a fixture by hand with the published-equivalent command, e.g. `testing-conventions unit coverage --language rust .github/selftest/integration-rust/clean`, since the PR's own CI won't exercise the post-release path.

A second #206 follow-up: zero-config Rust coverage also routed `packages/python` into the rust matrix, because `detect.has_rust_crate` matched a bare `Cargo.toml`. `packages/python` carries a `Cargo.toml` but generates its Rust sources at wheel-build time, so a plain checkout has no `.rs` — and the rust coverage/mutation jobs then ran `cargo` over absent sources and failed (`can't find … src/main.rs`). This stayed latent until a PR touched `testing-conventions.yml` (the only `dogfood.yml` trigger that re-runs the `packages/python` reusable-workflow call). The fix: `has_rust_crate` now requires a `Cargo.toml` **and** at least one `.rs` source, so a manifest with nothing to measure is not treated as a crate. Like any detection change, it only reaches the self-test / dogfood once a release moves `@v0`.

Each self-test job's assertion — run a published `npx testing-conventions` command over a fixture, then pass/fail on its exit code — lives as a standalone, colocated-tested script under `.github/scripts/` (epic #302). The failure-path jobs (#309) — `isolation-red`, `below-floor`, `mutation-gate`, `python-mutation-clean`, `packaging-red`, `coverage-rust-red`, `integration-lint-new-arms-trip`, `packaging-package-root-red` — each hold their hardcoded invocations in a `check_<name>.py` `CHECKS` list, decide pass/fail in a pure function, and carry colocated unit, boundary-mocked integration, and real-subprocess e2e tests. The workflow step runs `python3 .github/scripts/<name>/check_<name>.py`; the tested Python holds the invocation and the exit-code logic, so it earns the same dogfood gate as `detect.py` / `move_major_tag.py` and stays clear of the `${{ }}` templating trap an inline `run:` body carries.

## Rolling release: how `@v0` advances

`@v0` is a **moving major tag**: consumers pin `…/testing-conventions.yml@v0` and `…/actions/detect@v0`, and the tag is force-moved forward on each release so every consumer tracks `main`. We own all consumers and fix forward — this is rolling release, the opposite of a semver pin.

The tag is advanced by a dedicated workflow, `.github/workflows/move-major-tag.yml`, **not** inline in `release.yml`. It is **gated on a successful publish**: it triggers via `workflow_run` on the `Release` workflow completing and runs only when `conclusion == 'success'` (on `main`). That gate is the one place this repo departs from the generic "move the tag on every push to `main`" recipe, and it is non-negotiable:

The reusable workflow runs the **published** binary (`npx testing-conventions` → latest on npm), but the workflow *file* is frozen at `@v0`. If `@v0` advanced to a commit whose workflow invokes a subcommand the npm-latest binary doesn't expose yet (a rename/addition — the #55 class of break), every consumer running in the publish window would get new-workflow + old-binary → `unrecognized subcommand`. Publishing the binary is this repo's analog of committing a built `dist/`: ship the runtime first, then move the tag. `needs: release` (#92) did this inline; `move-major-tag.yml` does it as a named, single-responsibility workflow.

Two safety properties:

- **Concurrency** (`group: move-major-tag`, `cancel-in-progress: true`): the newest release wins; a stale in-flight move is cancelled.
- **Forward-only**: the tag moves to the released SHA only when that SHA is a descendant of the current `@v0` (otherwise it's a no-op), so out-of-order release runs can never rewind `@v0`. It also bootstraps `@v0` on first run.

The forward-only logic is a repo-only, pytest-covered helper — `.github/scripts/move-major-tag/move_major_tag.py`, behind a small git boundary so it carries integration tests (git mocked) and e2e tests (a real repo with a local remote), run by `move-major-tag-tests.yml` — exactly like the `detect` helper. The workflow YAML only wires the trigger, the checkout, and the env; it holds no logic.

The wiring is guarded in CI (`rolling-release-wired` in `testing-conventions-selftest.yml`): a regression that re-introduces an inline or un-gated tag move fails the self-test.

## Dogfooding the `.github/` helpers

The `.github/` helper scripts are first-party Python and are held to the same conventions as the shipped packages, not waved through: `dogfood-github-helpers.yml` runs `unit colocated-test`, `unit lint`, `unit coverage`, and `integration lint` whole-tree, plus the diff-scoped `unit mutation --base origin/main` (the published binary, via `npx`), over `.github/actions/` and `.github/scripts/`. So `detect.py` and `move_major_tag.py` carry a colocated unit test, stay isolated, meet the coverage floor, pass the mocking lint, and kill every mutant on the lines a PR touches — like any package source. Each helper keeps its colocated unit test next to the source and its integration/e2e suites under `tests/` (uniquely named, so `unit coverage`'s pytest collects them without an import-mode flag).

Mutation is diff-scoped, exactly like the reusable workflow's gate on the packages: whole-tree mutation is too slow to gate, so only survivors on `<base>...HEAD` changed lines count (`base` is `origin/main`; a run with no changed helper source reports no survivors and passes). The Python mutation arm is driven by the adapter shipped in the `testing-conventions` **wheel** (`python3 -m testing_conventions.mutation.main`, #248), so the job installs the wheel — which brings cosmic-ray — alongside pytest and coverage. The wiring is guarded in CI (`github-helpers-wired` in `testing-conventions-selftest.yml`): a regression that drops any of the five arms fails the self-test. That guard runs a standalone, colocated-tested script — `.github/scripts/github-helpers-wired/check_github_helpers_wired.py`, whose pure `find_missing_arm` names the first arm the dogfood workflow fails to invoke — rather than inline workflow bash (extracted per epic #302, sub-issue #310).

The self-test's own output assertions live here too, as standalone colocated-tested scripts under `.github/scripts/` rather than inline `run:` bash (epic #302). #308 moves the `detect` package-root checks: `detect-package-root-ts` and `detect-package-root-py` each run `./.github/actions/detect` against a monorepo fixture, then hand the resulting outputs — `package_root`, `ts_package_manager` / `python_env`, `provision_rust`, `config` — as CLI arguments to `check_detect_package_root_{ts,py}.py`, whose pure `evaluate` compares each against the value package-root discovery (#277) must produce. A colocated unit test drives `evaluate` in isolation, an e2e test runs `main` and the `__main__` guard via `runpy`, and the workflow step invokes the script directly, so the assertion carries the same coverage and mutation bar as the shipped source.

The self-test's wiring assertions live as standalone, colocated-tested scripts under `.github/scripts/` (epic #302). The package-root wiring checks — `coverage-package-root-wired`, `packaging-package-root-wired`, and `mutation-package-root-wired` (#307) — each read `testing-conventions.yml`, isolate the relevant job's YAML region, and confirm it references `needs.detect.outputs.package_root`. Carrying the logic as a first-party script gives it the colocated unit and e2e tests the dogfood gate enforces, and keeps the grep pattern in Python source where the GitHub Actions `${{ }}` templating pass never rewrites it.

The scan is scoped to the helper *code* by design. `.github/selftest/**` is **excluded** — those are intentional negative fixtures (below-floor suites, surviving mutants, un-colocated reds) the rules are *meant* to fire on — and the workflow YAML has nothing to scan. Pointing the rules at the repo root instead would need a real detection exclude/ignore (the root is full of negative fixtures under `packages/**/tests/fixtures/**` and `.github/selftest/**`, plus generated trees like `packages/python`'s build-time `.rs`, cf. #206); that mechanism is its own work, not what this gate does.

The self-test's own wiring/assertion checks live as standalone, colocated-tested scripts under `.github/scripts/` too (epic #302). Each selftest job that used to run an inline `grep` in a multi-line `run: |` block now invokes `python3 .github/scripts/<check>/check_<check>.py`, so the assertion carries a pure unit test (crafted wired/unwired strings) beside the source and an e2e test that runs `main` over a temp fixture — held to the same dogfood bar as the other helpers. The Python keeps the wiring pattern out of a `${{ }}`-templated string, where a literal `${{ … }}` would be evaluated before the shell ever saw it. Sub-issue #306 extracts the feature-input wiring checks: `build-command-wired`, `gates-wired`, and `rust-toolchain-wired`.
