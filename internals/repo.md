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

A third worked example, and a caution against over-attributing reds to `@v0` lag (#355): after the #351 `@v0` flip, `build-command-clean` and `rust-toolchain-clean` (the `[python].build_command` runtime fixtures, #243/#263/#289) still failed — `ModuleNotFoundError: No module named 'generated'`, the build step silently skipped. The workflow's own comments blamed the usual `@v0`/published-binary lag, but `@v0` was already current (it points at the same commit as `main`). The real cause: #335 generalized `build_command`'s config lookup to key off `primary_language(package_root)`, which returns `''` without a manifest (`pyproject.toml`/`package.json`/`Cargo.toml`) — but both fixtures are deliberately manifest-less (a bare pip Python package, #289's original case), so the lookup silently dropped the build step regardless of `@v0`. Fixed in `detect.compute_outputs`: `build_command`'s language falls back to the single present language when no manifest names a primary one (still empty, never guessed, when more than one language is present with no manifest to disambiguate). The lesson: a self-test red after a `@v0` flip is only actually *just* `@v0` lag if the *local* source (this PR's own `detect.py`, not the tag) also passes — check that first, per **Layer 1** in #353, rather than assuming the documented lag and waiting for the next release. Fixing the source doesn't make `build-command-clean` / `rust-toolchain-clean` green in *this* PR's own CI, though: `detect` here is still `actions/detect@v0`, so the fix only reaches this job once a release moves the tag — the ordinary pre-release lag, now with a real bug it had been masking underneath it.

Each self-test job's assertion — run a published `npx testing-conventions` command over a fixture, then pass/fail on its exit code — lives as a standalone, colocated-tested check (epic #302). The failure-path jobs (#309) — `isolation-red`, `below-floor`, `mutation-gate`, `python-mutation-clean`, `packaging-red`, `coverage-rust-red`, `integration-lint-new-arms-trip`, `packaging-package-root-red` — have moved into the `internals/checks` package as `tc-checks <name>` subcommands (#328): each holds its hardcoded invocations in a `CHECKS` list and hands them to the shared `run_checks` orchestrator (`checks/utils/`), which runs each invocation — or a single trailing command, the benign `true`/`false` e2e seam — and decides pass/fail through the pure `failure_reason`; colocated `cli_test.py`, `run_checks_test.py`, and `failure_reason_test.py` cover the logic while a sibling e2e suite drives the real subprocess boundary through `CliRunner`. The workflow step runs `uv run --project internals/checks tc-checks <name>`; the tested Python holds the invocation and the exit-code logic, so it earns the same dogfood gate as the rest of the checks package and stays clear of the `${{ }}` templating trap an inline `run:` body carries.
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

The `.github/` helper scripts are first-party Python and are held to the same conventions as the shipped packages, not waved through: `dogfood-github-helpers.yml` runs `unit colocated-test`, `unit lint`, `unit coverage`, and `integration lint` whole-tree, plus the diff-scoped `unit mutation --base origin/main` (the published binary, via `npx`), over `.github/scripts/`. So `move_major_tag.py` carries a colocated unit test, stays isolated, meets the coverage floor, passes the mocking lint, and kills every mutant on the lines a PR touches — like any package source. It keeps its colocated unit test next to the source and its integration/e2e suites under `tests/` (uniquely named, so `unit coverage`'s pytest collects them without an import-mode flag). `detect.py` used to be dogfooded here too, until #363 migrated it into `internals/detect/` — see "The detect action's package" below.

Mutation is diff-scoped, exactly like the reusable workflow's gate on the packages: whole-tree mutation is too slow to gate, so only survivors on `<base>...HEAD` changed lines count (`base` is `origin/main`; a run with no changed helper source reports no survivors and passes). The Python mutation arm is driven by the adapter shipped in the `testing-conventions` **wheel** (`python3 -m testing_conventions.mutation.main`, #248), so the job installs the wheel — which brings cosmic-ray — alongside pytest and coverage. The wiring is guarded in CI (`github-helpers-wired` in `testing-conventions-selftest.yml`): a regression that drops any of the five arms fails the self-test. That guard runs `uv run --project internals/checks tc-checks github-helpers-wired` — the check migrated into the `internals/checks` package (#329), whose pure `wires_github_helpers` is true only when the dogfood workflow invokes all five arms — rather than inline workflow bash (extracted per epic #302, sub-issue #310; consolidated per #321).

The self-test's own output and wiring assertions no longer live here — they have all migrated into the `internals/checks` package (see "The self-test checks package" below). This section now covers only `move_major_tag.py`, the one first-party helper still dogfooded as a loose script.

The scan is scoped to the helper *code* by design. `.github/selftest/**` is **excluded** — those are intentional negative fixtures (below-floor suites, surviving mutants, un-colocated reds) the rules are *meant* to fire on — and the workflow YAML has nothing to scan. Pointing the rules at the repo root instead would need a real detection exclude/ignore (the root is full of negative fixtures under `packages/**/tests/fixtures/**` and `.github/selftest/**`, plus generated trees like `packages/python`'s build-time `.rs`, cf. #206); that mechanism is its own work, not what this gate does.

## The self-test checks package (`internals/checks`)

The #302 wiring/assertion checks are consolidated into a single uv package at `internals/checks/` — `pyproject.toml` + `uv.lock` + a `src/checks/` layout (epic #321, complete). `checks/cli.py` is a `@click.group()` (`tc-checks`) that composes each check as a subcommand; each check lives in its own subpackage — `checks/<check>/cli.py` holds a pure predicate (or, for the failure-path group, a hardcoded `CHECKS` list) and a `@click.command()`, with a colocated `cli_test.py`. Shared code lives in `checks/utils/`: `check_failed.py` (the `CheckFailed` `click.ClickException` that prints a `::error::` annotation), `run_checks.py` + `failure_reason.py` (the failure-path orchestrator and its exit-code decision), and `job_block.py` (isolating a named job's YAML region). A self-test job runs `uv run --project internals/checks tc-checks <check>`.

The layout mirrors `packages/python`, whose importable package sits in `packages/python/python` while `packages/python/tests` holds the integration/e2e suite: `path` for the dogfood points at the **inner** `internals/checks/src`, not the package root, so the static gates recurse only the source tree. The colocated `cli_test.py` units drive each check's `@click.command` through its `.callback` (no `CliRunner`, which is a third-party collaborator the isolation lint flags) and import only the unit under test — so the colocated suite alone reaches the 100% coverage floor. The full e2e suite (`CliRunner` over the real workflow file) lives at `internals/checks/tests/e2e`, a sibling **outside** the scanned `src/`; a `*_test.py` e2e file *inside* the scan would be read as an un-isolated unit test and red the lint. The package root (`internals/checks`, where the `pyproject.toml` lives) is still derived for the coverage/mutation venv.

The packaging gate's `packaging_build` derivation covers `internals/checks` too (a plain `uv build`, #335), so the dogfood packaging job builds this package's own distributions and scans them — and both must exclude the colocated `*_test.py` units the same way any other zero-config Python package would, or the scan rejects the artifact as shipping its tests (#354). `uv build` produces a wheel *and* an sdist, and hatchling's `[tool.hatch.build.targets.wheel]` / `[tool.hatch.build.targets.sdist]` exclude independently of each other — an exclude scoped to only the wheel target leaves the sdist (`.tar.gz`) shipping every test file untouched. The top-level `[tool.hatch.build] exclude = ["**/*_test.py"]` applies to both targets at once. Tests still run from the source tree (`.venv`/`uv run pytest`), never from a built artifact, so the exclude has no effect on execution — only on what `uv build` packages.

It lives under `internals/`, **not** `.github/scripts/`, on purpose: `dogfood-github-helpers.yml` scans `.github/scripts/` as *loose* first-party scripts, and a `pyproject.toml` inside that scan flips the conventions tool into package-mode for the whole directory (it stops recognizing the loose scripts' `tests/integration` / `tests/e2e` as non-unit). As a real package it is instead dogfooded through the **shipped reusable workflow** (`dogfood.yml`, `path: internals/checks/src`) — colocated-test, isolation, coverage, integration-lint, and diff-scoped mutation — exactly like `packages/python`. `dogfood-github-helpers.yml` now covers exactly one genuinely loose helper — `move_major_tag.py` (every #302 check has migrated into the package, and `detect.py` into `internals/detect`), so `.github/scripts/` holds a single directory.

## The detect action's package (`internals/detect`)

`detect.py` (the `detect` composite action's implementation, #189/#277 onward) moved out of `.github/actions/detect/` into its own uv package, `internals/detect/` (#363), mirroring `internals/checks` for the same reason: custom logic under `.github/` is loose-script territory, and adding a `pyproject.toml` there would have flipped `dogfood-github-helpers.yml`'s scan into package-mode. `internals/detect/src/detect.py` is a single top-level module (no subpackage — one file, no CLI subcommands to compose), with its colocated `detect_test.py` beside it and the integration/e2e suites at `internals/detect/tests/`, a sibling outside `src/`, exactly like `internals/checks`.

Unlike `internals/checks`, it is **not** dogfooded through the shipped reusable workflow. `internals/checks`' colocated `cli_test.py` units alone reach the coverage floor, so scoping the dogfood job's `path` to `src/` (excluding the e2e suite entirely) works cleanly. `detect.py`'s colocated `detect_test.py` alone does not — `compute_outputs`'s orchestration is exercised only by the integration suite (filesystem mocked) and the full script only by the e2e suite. Scoping to `internals/detect/src` alone therefore fails the coverage floor (the integration/e2e suites are outside the scan and never run); scoping to the package root instead (`internals/detect`, so all three tiers run together) fails `unit lint`'s `unmocked-collaborator` rule, because that rule has no concept of test tiers — once a first-party package is declared (any `pyproject.toml`), it flags *every* `*_test.py` under the scanned root that imports the package unmocked, `detect_integration_test.py` and `detect_e2e_test.py` included. (This also explains why `detect.py` silently passed `dogfood-github-helpers.yml`'s isolation check for years despite the same nested layout: `.github/actions` never had a `pyproject.toml`, so the rule's first-party-package lookup found nothing and reported no violations at all — not because the layout satisfied it.) `detect.py` keeps its existing, proven test-quality gate instead: `detect-action.yml`'s dedicated pytest run across all three tiers together (100% coverage via plain `coverage.py`), independently of this tool's own gates.

`.github/actions/detect/action.yml` is unaffected by the move — it is a thin composite-action manifest, not Python, and it is the file every consumer's `uses: …/actions/detect@v0` reference resolves against. Its `run:` step now points at `$GITHUB_ACTION_PATH/../../../internals/detect/src/detect.py`: GitHub Actions checks out the *whole* repo at the pinned ref to resolve a composite action (not just the action's own subdirectory), so a relative path climbing back out to the repo root and down into `internals/` resolves identically whether the action is used locally (`./.github/actions/detect`) or externally (`owner/repo/.github/actions/detect@ref`). The `uses:` contract itself never changes, so this is not a breaking change for any consumer and needs no `MIGRATIONS.md` entry.

With epic #321 complete, every #302 wiring/assertion and failure-path check lives in `internals/checks` as a `tc-checks <check>` subcommand; the flat `.github/scripts/<check>/` dirs are gone, and each self-test job invokes `uv run --project internals/checks tc-checks <check>` after `astral-sh/setup-uv`. The full inventory, by original sub-issue:

- **Wiring assertions (#323):** `mutation-wired`, `isolation-wired`, `coverage-rust-wired`, `colocated-rust-wired`, `diff-scoped-wired`, `e2e-verify-wired`, `e2e-verify-checks-out-pr-head` (block-scoped to the `e2e-verify` job, replacing the old `awk` range), `e2e-verify-scope-wired`, `rolling-release-wired` (two selftest steps folded into one command over two file arguments).
- **Detect wiring (#324):** `wiring-detect-action`, `wiring-packaging-default-on`, `wiring-e2e-default-on`, and `detect-routes-python` — the last keeps its `uses: ./.github/actions/detect` step in the job and passes the action's `isolation_languages` output as a single-quoted JSON CLI argument.
- **Feature-input wiring (#325):** `build-command-wired`, `gates-wired`, `rust-toolchain-wired`.
- **Package-root wiring (#326):** `coverage-package-root-wired`, `packaging-package-root-wired`, `mutation-package-root-wired` — each isolates a job's YAML region and asserts it references `needs.detect.outputs.package_root`.
- **Detect-output validations (#327):** `detect-package-root-ts`, `detect-package-root-py` — each runs `./.github/actions/detect` against a monorepo fixture and hands the outputs to a pure `evaluate` returning the first mismatch's message.
- **Failure-path (#328):** `isolation-red`, `below-floor`, `mutation-gate`, `python-mutation-clean`, `packaging-red`, `coverage-rust-red`, `integration-lint-new-arms-trip`, `packaging-package-root-red` — each runs published `npx testing-conventions` invocations from a `CHECKS` list and asserts the exit code via `failure_reason`.
- **github-helpers-wired (#329).**

The static checks hold their inspection in a pure predicate over the workflow file; the failure-path group holds a `CHECKS` list run through the shared `run_checks` orchestrator. Either way the colocated `cli_test.py` drives the pure logic in isolation, the `@click.command()` raises `CheckFailed` (a `::error::` annotation) on a failure, and a sibling `CliRunner` e2e suite exercises the real boundary — held to the same coverage and mutation bar as any shipped source.

The two pre-existing first-party helpers were resolved per the #321 open question: `detect.py` moved to `internals/detect` (#363), while `move_major_tag.py` stays a loose script under `.github/scripts/` — it wires the tag-move workflow, has no CLI subcommands to compose into the checks group, and is already held to the full bar by `dogfood-github-helpers.yml`. `.github/scripts/` therefore holds exactly one directory, and the epic is closed.

## Rust CI: nextest, and why the coverage job's cache needed no change (#370)

`rust.yml`'s `integration` job ("Integration + e2e tests + coverage (95%)") runs the ~65 files under `packages/rust/tests/` through `cargo llvm-cov`. #370 (epic #366) asked for two things: a reliable, distinct cache for the coverage-instrumented build, and running under `nextest`. Only the second turned out to be real.

**The cache ask was already satisfied.** `Swatinem/rust-cache@v2` bakes the GitHub Actions job name into its default key, so `lint`, `unit`, `integration`, and `build` already get four separate, non-colliding caches — confirmed by inspecting live cache-key strings in CI logs (`v0-rust-integration-Linux-x64-…` vs `v0-rust-lint-Linux-x64-…`). This separation isn't incidental: `cargo llvm-cov` compiles under `-C instrument-coverage` into a distinct `target/llvm-cov-target/` directory, so its build artifacts could never usefully share a cache with the other jobs' plain `cargo build`/`cargo test` output regardless of key tuning. The actual (occasional) cache misses trace to `dtolnay/rust-toolchain@stable` being unpinned — a rustc point-release bump invalidates all four jobs' caches simultaneously — but pinning it was out of scope here: the same action is used seven more times in the *shipped* reusable workflow (`testing-conventions.yml`), and pinning there is a consumer-facing toolchain-provisioning decision with its own maintenance cost, not an internal CI tweak. Cold-vs-warm compile time for this job also turned out to be a modest 15–30% gap in practice, not the dominant cost — so no cache changes were made.

**`nextest` is the real fix.** The 65 integration-test files each compile to their own binary; the default harness runs them one at a time, and several cost multiple seconds to tens of seconds because they shell out to real subprocesses (pytest, `npx vitest`, `cargo-mutants`) — that serial cost, not compilation, is what dominates the job's wall clock. `cargo llvm-cov nextest --ignore-filename-regex 'main\.rs' --fail-under-lines 95` is a direct drop-in for the previous `cargo llvm-cov --ignore-filename-regex 'main\.rs' --fail-under-lines 95`: `--fail-under-lines`/`--ignore-filename-regex` are `cargo-llvm-cov`'s own report-gating flags, applied identically regardless of which test-runner subcommand executes the tests. The crate has zero doctests, so nextest's well-known "doesn't run doctests" gap costs nothing here.

One correctness question was worth answering empirically before landing this, not assuming: `mutation.rs`'s `ensure_cargo_mutants()` provisions a shared, version-scoped binary cache (`~/.cache/testing-conventions/cargo-mutants-<version>`) with no file locking — a bare "does the binary exist, if not run `cargo install`" check. nextest runs each test *binary* in its own OS process, in parallel, which could mean several processes racing to provision that shared cache simultaneously on a cold cache. Verified locally: cleared the cache, ran the mutation-Rust tests (which already run concurrently *within* one binary today, via the default harness's thread-per-test model — the same race shape, just within-process rather than cross-process) — all provisioning calls landed on an intact, correctly-sized binary, no corruption. Since `ensure_cargo_mutants()` always spawns a *fresh OS process* for the actual `cargo install` (`Command::new("cargo")`), the concurrency structure at the point that matters is identical whether the caller is a thread within one nextest-run binary or a separate process nextest itself spawned — so nextest's cross-binary parallelism doesn't introduce a new risk here, only exercises a path that was already exercised.
