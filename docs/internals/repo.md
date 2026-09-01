# Repo-wide conventions

Cross-cutting rules that apply across all language packages. Language-specific guidance lives in `python-supervision.md`, `typescript-supervision.md`, `rust-supervision.md`.

## CHANGELOG + MIGRATIONS

Each package records its changes as **fragments** — one file per change, added under
`packages/<pkg>/changelog.d/` and `packages/<pkg>/migrations.d/`. The convention below is global;
every language package follows it.

A PR that changes public API under `packages/<pkg>/` adds one fragment to each of that package's
two fragment directories. Enforced in CI by `changelog.yml`, which runs the `changelog-gate` check
(`internals/checks`); a `skip-changelog: <reason>` line on any commit bypasses it for genuinely
internal refactors, and the reason stays in git history.

**Why fragments.** A shared file that every PR appends to at the same anchor makes concurrent PRs
conflict by construction: the gate requires each PR to edit it, and the convention puts each new
entry at the top, so any two open PRs against one package collide. A fragment is a new file, so
two PRs touching the same package produce no conflict.

**Naming.** `packages/<pkg>/<kind>.d/YYYY-MM-DD-<slug>.md`, where `<kind>` is `changelog` or
`migrations`, the date is the UTC merge date, and `<slug>` is kebab-case (lowercase letters,
digits, hyphens). The directory names the package and the kind, so the filename carries neither.
Each fragment directory holds a `README.md` describing the convention; it is not an entry.

**Changelog fragment** — Keep a Changelog categories. The body opens with a bold category —
`**Added**` / `**Changed**` / `**Deprecated**` / `**Removed**` / `**Fixed**` / `**Security**` —
then the entry text as it would read in a changelog. The category lives in the body, not the
filename. A breaking change carries a `**BREAKING**` prefix and names its migration fragment.

**Migration fragment** — a `### <title>` heading and five sections, in order:

1. **Summary** — one paragraph: what changed and why.
2. **Required changes** — before/after for config, CLI flags, function/method arguments, action inputs.
3. **Deprecations removed** — anything previously warned about that's now gone.
4. **Behavior changes without code changes** — same API, different runtime behavior (tag format, exit codes, defaults).
5. **Verification** — commands the consumer runs to confirm the upgrade worked, with the expected output.

Keep every heading; write `_None._` under one that does not apply.

**Fragments are permanent and append-only.** Nothing is assembled back into a single file and
deleted, and no `## Unreleased` section is ever renamed — the release record is the git tag history
and the GitHub Release for each version. The package-root `CHANGELOG.md` and `MIGRATIONS.md` are a
**frozen archive** of the entries written before this convention: read them for history, never
append to them. Direction of travel is one way — an entry becomes a fragment, never the reverse.
If you are adding lines to `CHANGELOG.md` or `MIGRATIONS.md`, stop: you want a fragment.

Public-API surface for the purpose of these files: every exported value/type, every CLI flag, every config key, every observable artifact (tag format, GitHub Release body shape). Internal refactors, test-only changes, and docs-only edits stay out.

## The CLI command surface

Every subcommand `--help` lists runs a rule and can fail. A command that parses and exits `0`
without doing work hands a consumer a pass they never earned, and it reads as a documented feature
because it sits in the help output next to real ones. The `check` umbrella was the one such
scaffold — declared in the repo's first commit, dispatched into the no-subcommand arm, and left
unwired when #56 closed as not planned — and it is gone; nothing replaces it. A new command lands
wired.

Two shapes sit outside that rule, both deliberate:

- **`workflow`** is hidden (`#[command(hide = true)]`) rather than absent. It does real work — the
  drift guard that walks a workflow file's invocations against the binary's own command tree — but
  it runs from this repo's CI, not from a consumer's, so it stays out of the documented surface.
- **A bare `testing-conventions`**, with no subcommand at all, prints the version banner on stderr
  and exits `0`. Nothing to run, nothing to report.

The command tree (`testing_conventions::command()`) is also the workflow guard's source of truth,
so removing a subcommand is a two-sided edit: the variant in `packages/rust/src/lib.rs`, and every
guard fixture under `packages/rust/tests/fixtures/workflow/` that names it. A clean fixture must
invoke commands the binary still exposes, or the fixture stops meaning "clean" and starts meaning
"stale."

## Monorepo package-root derivation

`detect.py` derives seven outputs a suite-executing job needs to install, build, run, and
configure at the right directory (#277, #289, #475): `package_root`, `ts_package_manager`,
`ts_pnpm_version`, `python_env`, `provision_rust`, `config`, `build_command`. A `working_directory` input was considered and
rejected — it would
add a second, consumer-facing coordinate system against the documented rule that `source` is the
only scoping mechanism (docs/monorepo.md). Everything else is derived from `source` and the
package's own manifest instead.

- **`package_root`** (`derive_package_root`): the nearest directory at-or-above the scan root
  (`source`), down to the checkout root inclusive, holding a `package.json` / `pyproject.toml` /
  `Cargo.toml`; the checkout root (`.`) when none is found. A single-package repo has no manifest
  above the scan root other than possibly at the checkout root, so it always derives `.` —
  every current consumer is untouched.
- **`ts_package_manager`** (`ts_package_manager`): `package.json`'s `packageManager` field name,
  else `pnpm`/`npm` by lockfile presence, else `pnpm` (today's hardcoded default).
- **`ts_pnpm_version`** (`ts_pnpm_version`): the `version` input the reusable workflow hands
  `pnpm/action-setup` — `packageManager`'s own pin when it names pnpm, else the `>=11` floor.
  `action-setup` throws `Multiple versions of pnpm specified` whenever `version` is set and
  `packageManager` is not string-equal to it, which no range ever is, so echoing the pin back is the
  only non-empty value it accepts; it resolves to `pnpm@<pin>`, exactly what passing nothing would
  install (#475). Never empty, which is what lets the workflow read empty as "this detect predates
  the output" and fall back to the floor — a rolling `@v0` gates each release on the *published*
  detect, so every new output spends one release absent. This repo pins floors through
  `engines` and carries no `packageManager` field, so the conflicting path is invisible to
  dogfooding — the selftest fixture covers it instead.
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
- **`build_command`** (`derive_build_command`, #289/#335): the `[<language>].build_command` shell
  command read from the discovered `config` file, keyed off the package's `primary_language` — so
  `[python]`, `[typescript]`, and `[rust]` are all read. This is the only detect function that
  opens and parses a `testing-conventions.toml`'s *contents*, not just resolves its path; `''` when
  the file is absent, unparseable, or declares no build command. The suite-executing jobs run
  `needs.detect.outputs.build_command` before the suite and the packaging job runs it before the
  pack, replacing the removed `build_command` *workflow input*.

Alongside the package-root set, detect emits one **language set per gate** — the JSON arrays each
matrix reads back with `fromJSON`, kept separate so a future per-gate language divergence needs no
workflow change:

- **`languages`** — the requested python/typescript languages with sources under `source`; the
  co-change (`*-changed`) matrix. Rust units are inline, so a sibling test cannot go stale.
- **`colocated_test_languages`**, **`integration_lint_languages`**, **`isolation_languages`**,
  **`static_languages`**, **`one_function_languages`** — the same set plus rust. The five static
  gates run as steps of one `static` job fanned out over `static_languages`; an unconfigured rust
  tree reports `one-function-per-file` is off and passes, so rust rides that set too.
- **`coverage_languages`**, **`mutation_languages`** — present python/typescript, plus rust
  whenever a crate is present. Rust coverage is zero-config (`lines = 100` by default), and all
  three mutation arms are at parity, so the two sets are identical today.

It also emits two presence flags the packaging and e2e-verify jobs gate on — **`packaging_dist`**
(a built distribution is discoverable at `package_root`) and **`e2e_attestation`** (committed
receipts sit in `e2e-attestations/` there) — so both gates run by default and skip, never fail,
when absent.

**`cargo_target_dir`** (#410) is the workspace-aware Rust build-cache location: the workspace
root's `target/` for a workspace-member crate, else the package root's own. cargo resolves the
target directory at the workspace root regardless of the invoking directory, so a cache keyed on
`package_root` alone would archive and restore a directory cargo never writes to.

Each language-set output spends one release absent, exactly as `ts_pnpm_version` describes above,
so the workflow's matrix expressions carry a `|| <older set>` fallback until `@v0` advances.

These are the primitive the four gate fixes (#278–#281) consume; deriving them is out of scope
for what those jobs *do* with them (installing, building, discovering `dist/`, discovering
e2e receipts) — see each issue for its own gate-specific wiring.

The `.github/selftest/monorepo/` fixture (no manifest or lockfile at its own root, mirroring a
real per-package-lockfile monorepo) exercises the derivation end to end via the local
`./.github/actions/detect` action — the same pattern `detect-routes-python` in
`testing-conventions-selftest.yml` already uses, so it isn't blocked by the `@v0` lag described
below.

**Writing the outputs to `GITHUB_OUTPUT` (#396).** `main` renders the outputs through
`render_github_output` before appending them to the `GITHUB_OUTPUT` file. A single-line value is a
plain `name=value` line; a value that carries a newline — a `build_command` declared as a TOML
`"""…"""` multi-line string — is written in the runner's heredoc form (`name<<DELIM`, the value,
then `DELIM` on its own line), with a content-derived delimiter that can't collide with any line of
the value. A raw `name=value` line for a multi-line value would corrupt the file: the embedded
newline ends the file-command line early, and the value's remaining lines are parsed as further
(bogus) outputs. `render_github_output` is a pure function with its own colocated test, so the
rendering is exercised in isolation, not only through a full action run.

## Self-test and the `@v0` path

`testing-conventions-selftest.yml` smoke-tests the reusable workflow end to end, so a regression in
its *wiring* — a renamed input, a broken invocation, a dropped toolchain step — surfaces here rather
than in a consumer repo. Rule *logic* is covered by the Rust e2e suites (`coverage_e2e.rs`,
`coverage_ts_e2e.rs`, …); this covers the workflow that drives it.

Its jobs follow a three-name convention, and a rule earns as many of the three as it has surface:

- **`<rule>-wired`** — a `tc-checks` static assertion over the workflow file, so it tracks the
  wiring regardless of what the published binary ships.
- **`<rule>-clean`** — a passing fixture driven through a real `uses:` call of the reusable
  workflow; the whole call must pass, which also proves the fixture is clean for every *other* gate
  it gets fanned over.
- **`<rule>-red`** — a violating fixture driven through the CLI directly, asserting a non-zero exit.
  The red path cannot ride a `uses:` call, because a failing call fails the whole run.

The fixtures live under `.github/selftest/`, and the `-red` jobs drive the hermetic binary
(`./hermetic-cli/testing-conventions`) rather than npm-latest (#379, below). Two fixture trees are
worth knowing about: `.github/selftest/monorepo/` carries no manifest or lockfile at its own root,
the per-package-lockfile shape the package-root derivation runs against, and
`.github/selftest/packaging-package-root/` is generated — regenerate it with `python
.github/selftest/packaging-package-root/make_fixtures.py`.

The reusable workflow (`.github/workflows/testing-conventions.yml`) drives the **published** tool — its `detect` step pins `…/actions/detect@v0`, and each rule job runs `npx testing-conventions` (no version → latest on npm). The self-test (`testing-conventions-selftest.yml`) calls that reusable workflow. So a change to *detection* (which rules fan out) or *rule behavior* does **not** take effect in the self-test — or for any consumer — until a release **moves `@v0`** to the new commit and publishes the package.

The trap: a change can stay green in its own PR's self-test (still running the old `@v0` path) yet break the self-test on the **next release**, when `@v0` advances. So any change that alters which rules a fixture is fanned over must leave every self-test fixture passing under the *new* path, not just the merged one. Concretely, a fixture driven through the reusable workflow (`uses:`) must pass **every** rule it could be fanned over — not only the rule it was added for.

**As of #353, this trap is closed mechanically — it is no longer a rule the author must remember.** The CI-hermeticity invariant (AGENTS.md, "CI hermeticity: a required check depends only on the commit under test") is enforced in two layers, each with its own section below:

- **Layer 1 — the merge gate now runs HEAD hermetically (#356, "Hermetic mode").** Every PR's self-test and dogfood build `detect` and the CLI from HEAD and run *those*, so a detection or rule-behavior change is validated against the commit under test in its own PR — `(HEAD workflow × HEAD detect × HEAD-built binary)`, the frozen `@v0`/npm-latest references replaced by HEAD end to end. A change that would only have surfaced "on the next release" now goes red in the PR that introduces it.
- **Layer 2 — consumer-surface validation moved to the gated `@v0` promotion (#357, "Validated promotion").** The one input Layer 1 structurally can't pin — the frozen `@v0` a consumer runs the instant the tag moves — is validated at promotion instead: the full self-test + dogfood surface runs pinned to the just-published immutable version, and `@v0` advances only if green (fail closed).

The #351 packaging flip is the cautionary case that motivated both (its own worked example is the third one below): green in its PR's self-test under the old `@v0`, red on `main` only when the next release advanced the tag — the exact "green gate that tested the wrong thing" the two layers now catch, in-PR for HEAD-buildable skew and at promotion for the published surface. The worked examples that follow predate the two layers and record the incidents that drove them; the manual discipline they describe ("verify a fixture by hand with the published-equivalent command") is now the gate's job, not the author's.

Worked example (#206): making Rust coverage zero-config routed every detected Rust crate into the coverage matrix. The lint-only `integration-rust/clean` fixture then had to become coverage-clean too — its integration test runs first-party code for real (and so compiles under `cargo llvm-cov`) rather than carrying a `#[double]` that only ever parsed for the lint. A second round (#265): scoping the Rust coverage arm to the unit suite (`--lib`) took the integration tier out of the number, so the fixture also carries an inline `#[cfg(test)]` test that covers `compute` — the unit suite clears the floor on its own. Verify a fixture by hand with the published-equivalent command, e.g. `testing-conventions unit coverage --language rust .github/selftest/integration-rust/clean`, since the PR's own CI won't exercise the post-release path.

A second #206 follow-up: zero-config Rust coverage also routed `packages/python` into the rust matrix, because `detect.has_rust_crate` matched a bare `Cargo.toml`. `packages/python` carries a `Cargo.toml` but generates its Rust sources at wheel-build time, so a plain checkout has no `.rs` — and the rust coverage/mutation jobs then ran `cargo` over absent sources and failed (`can't find … src/main.rs`). This stayed latent until a PR touched `testing-conventions.yml` (the only `dogfood.yml` trigger that re-runs the `packages/python` reusable-workflow call). The fix: `has_rust_crate` now requires a `Cargo.toml` **and** at least one `.rs` source, so a manifest with nothing to measure is not treated as a crate. Like any detection change, it only reaches the self-test / dogfood once a release moves `@v0`.

A third worked example, and a caution against over-attributing reds to `@v0` lag (#355): after the #351 `@v0` flip, `build-command-clean` and `rust-toolchain-clean` (the `[python].build_command` runtime fixtures, #243/#263/#289) still failed — `ModuleNotFoundError: No module named 'generated'`, the build step silently skipped. The workflow's own comments blamed the usual `@v0`/published-binary lag, but `@v0` was already current (it points at the same commit as `main`). The real cause: #335 generalized `build_command`'s config lookup to key off `primary_language(package_root)`, which returns `''` without a manifest (`pyproject.toml`/`package.json`/`Cargo.toml`) — but both fixtures are deliberately manifest-less (a bare pip Python package, #289's original case), so the lookup silently dropped the build step regardless of `@v0`. Fixed in `detect.compute_outputs`: `build_command`'s language falls back to the single present language when no manifest names a primary one (still empty, never guessed, when more than one language is present with no manifest to disambiguate). The lesson: a self-test red after a `@v0` flip is only actually *just* `@v0` lag if the *local* source (this PR's own `detect.py`, not the tag) also passes — check that first, per **Layer 1** in #353, rather than assuming the documented lag and waiting for the next release. Fixing the source doesn't make `build-command-clean` / `rust-toolchain-clean` green in *this* PR's own CI, though: `detect` here is still `actions/detect@v0`, so the fix only reaches this job once a release moves the tag — the ordinary pre-release lag, now with a real bug it had been masking underneath it.

Each self-test job's assertion — run a CLI command over a fixture, then pass/fail on its exit code — lives as a standalone, colocated-tested check (epic #302). The failure-path jobs (#309) — `isolation-red`, `below-floor`, `mutation-gate`, `python-mutation-clean`, `packaging-red`, `coverage-rust-red`, `integration-lint-new-arms-trip`, `packaging-package-root-red`, and `colocated-rust-red` (#379) — have moved into the `internals/checks` package as `tc-checks <name>` subcommands (#328): each holds its hardcoded invocations in a `CHECKS` list and hands them to the shared `run_checks` orchestrator (`checks/utils/`), which runs each invocation — or a single trailing command, the benign `true`/`false` e2e seam — and decides pass/fail through the pure `failure_reason`; colocated `cli_test.py`, `run_checks_test.py`, and `failure_reason_test.py` cover the logic while a sibling e2e suite drives the real subprocess boundary through `CliRunner`. The workflow step runs `uv run --project internals/checks tc-checks <name>`; the tested Python holds the invocation and the exit-code logic, so it earns the same dogfood gate as the rest of the checks package and stays clear of the `${{ }}` templating trap an inline `run:` body carries. Each `CHECKS` list holds the **hermetic** binary (`./hermetic-cli/testing-conventions`), shared from `checks/config.py`'s `HERMETIC_CLI`, and each red-path job downloads the `hermetic-cli` artifact (`needs: [build-cli]` + `./.github/actions/download-hermetic-cli`) so it validates this branch's CLI, not npm-latest (#379) — the `red-path-hermetic-wired` check gates that wiring.

## CI provisions from disk: uv, and the source mutation adapter (#352)

Inside CI jobs the Python toolchain comes from **uv, and this repo's own mutation adapter comes from the source tree** — never `pip install`, and never a fetch of the published `testing-conventions` wheel. Two separable facts sit behind that one rule:

- **The engines are third-party, and each is a pinned dependency of the package whose job runs it.** `coverage`, `pytest`, `cosmic-ray`, and `maturin` live nowhere in this repo, and an engine resolved from index-latest at run time is a mutable external reference inside a required check — a new engine release can red the check, or change the shipped wheel, with no commit to blame (AGENTS.md, "CI hermeticity"). So an engine is declared where a dependency belongs: in the owning uv package's dev dependency group, pinned in that package's existing `uv.lock`, with the job running through the project (`uv run --project <pkg> …`) so the version a run resolves is a function of the commit. Bumping an engine is a committed `uv lock --upgrade` diff, and setup-uv's cache (keyed on the lock) reuses the environment across runs. Where that stands per engine site:
  - `internals/detect` — `pytest` (its only engine; `detect.py` is stdlib-only) is a dev-dependency pinned in its lock, and `detect-action.yml` runs the suite from the package directory (#445).
  - `internals/checks` — `coverage`, `pytest`, and `cosmic-ray` are dev-dependencies pinned in its lock; the selftest jobs that shell out to them (`below-floor`, `python-mutation-clean`) already run `--project internals/checks` and pick them up from the sync. The one job with no uv project of its own (`rust.yml` integration — a Rust crate) **borrows** that environment (#446), syncing it once and putting its `.venv/bin` on `PATH`: a deliberate coupling, accepted because the checks package is the repo's test-tooling home; if a cleaner home emerges for the borrowing job, prefer it.
  - the `packages/python` wheel build (`python.yml`) — `maturin` is not a test engine but the PEP 517 build backend, so its pin lives where a build backend is declared: `[build-system].requires`, pinned exactly (#448). The job builds through it (`uv build --wheel`), so the toolchain that builds the shipped wheel is a function of the commit.
  - the reusable workflow's suite jobs (the consumer path) — **the consumer's own lock is the pin** (#438). No lock of this repo's can reach that runner (the workflow executes in the consumer's checkout, and published wheels carry ranges, not locks), so the provisioning line installs only `pytest` (the runner fallback — a synced uv project's own locked pytest is already satisfied and wins) plus the `testing-conventions` wheel, whose dependencies carry the coverage/mutation engines. The float that remains — the wheel's ranges, for lock-less consumers — is the consumer-path exception below, gated at promotion (Layer 2).

- **The adapter is this repo's code, so it resolves from source, not the wheel.** `unit mutation --language python` spawns `python3 -m testing_conventions.mutation.main` (#248). Installing the published `testing-conventions` wheel to supply that module — `pip install testing-conventions`, or `uv run --with testing-conventions` — runs the *last release's* adapter over this PR's fixtures, the same `@v0`/npm version-skew class the hermetic merge gate (#356) closes for the CLI binary. The `hermetic-cli` artifact stages the Rust binary and the node `dist/`, not the Python adapter, so it does not reach this. The fix, which `rust.yml`'s integration job already models, is `PYTHONPATH: ${{ github.workspace }}/packages/python/python` — the adapter's source tree, ahead of any installed copy — so the adapter under test is the PR's. `cosmic-ray` (its engine, otherwise pulled in transitively by the wheel) then comes from `internals/checks`'s synced environment, pinned in its lock.

The consumer-path surface is the deliberate exception, and only for the artifact it validates: `python.yml` still `pip install`s the built `dist/*.whl` to load the pytest plugin the way an installed consumer does, and the reusable workflow's own jobs run the published binary for consumers. The direct-drive red-path self-test jobs route through `hermetic-cli` rather than ad-hoc `npx` (#379), and the reusable workflow's own Python provisioning is uv-only (#399).

Every job that provisions uv pins `astral-sh/setup-uv@v7`, whose bundled Node 24 runtime runs the action natively on the GitHub-hosted runner.

## Hermetic mode: building detect + the CLI from HEAD (#356)

Every job in the reusable workflow resolves two mutable external references at run time: the
`detect` step pins `…/actions/detect@v0` (a floating tag), and every rule job runs
`npx -y "testing-conventions${VERSION:+@$VERSION}"` (no version → latest on npm). For a
*consumer*, that's the whole point — they want the released, supported surface. But when
`testing-conventions.yml` gates its **own** merges (self-test, dogfood), it means a PR's own CI
validates `(this commit's workflow) × (whatever @v0/npm currently are)`, not the commit under
test — the exact skew #353 exists to close (worked examples: #206, #351, #355).

Hermetic mode is **derived, never declared — and the derivation lives in tested code, not YAML**.
The reusable workflow passes detect two facts it alone knows: `caller_repository`
(`${{ github.repository }}`, which for a reusable workflow always belongs to the **calling** run)
and `version` (`${{ inputs.version }}`). detect.py's `hermetic()` decides:

    caller_repository == 'thekevinscott/testing-conventions' and version == ''

An external consumer's call carries their own repository and can never match; an explicit
`version` always names the published artifact (which is what #357's post-publish verification
does) and wins even in-repo. When the derivation holds, detect emits `cli_command`
(`./hermetic-cli/testing-conventions`) and `ts_mutation_adapter_args` (the `--ts-mutation-adapter`
argument the npm launcher normally appends, pre-rendered like `e2e_extra_scope`, #333); for every
other caller both are empty. (Rejected alternatives, in order: a `hermetic` boolean input —
`workflow_call` inputs have no visibility modifier, so a testing-only flag is public surface any
consumer can flip; and a guarded `build-cli` job inside the reusable workflow — a job with a
false `if:` still renders a skipped row in every consumer's checks UI. The `hermetic-wired` check
fails on either reappearing.)

**The build lives in the callers.** `testing-conventions-selftest.yml` and `dogfood.yml` — repo-
only files no consumer references — each carry a `build-cli` job that checks out the repo and
calls the shared `./.github/actions/build-hermetic-cli` composite action for everything else:
provision rust/pnpm/node/uv, build the release binary from HEAD (the same binary
`packages/node/scripts/build.ts` stages for the npm packages) and `packages/node`'s `dist/` (the
TS mutation adapter) via the colocated-tested `tc-checks build-hermetic-cli` (internals/checks),
and stage both as the `hermetic-cli` artifact — binary at the artifact root, `dist/` beside it,
exec bit restored on download (artifact transfer drops it). One composite action, `uses:`'d by
both callers, so the two builds can't drift; `hermetic-wired` asserts each caller's `build-cli`
job calls it rather than inlining the steps. Every `uses:` call of the reusable workflow in those
files declares `needs: [build-cli]`: a called reusable workflow runs inside the caller's run, so
its jobs start only after the caller's `needs` are met and share the run-scoped artifact store.
One build per run, shared by every call.

**The reusable workflow carries only the consumption side, all step-level** (steps render no
checks rows, so a consumer's checks UI is unchanged):

- The `detect` job declares a step pair — `scan_hermetic` (`uses: ./.github/actions/detect`,
  HEAD's action: the caller's checkout IS this repo whenever the guard holds) and
  `scan_published` (`…/detect@v0`) — selected by the guard literal as step `if:`s. This is the
  one place the guard stays in YAML: which action ref runs is a scheduling decision only an
  expression can make (`uses:` cannot be dynamic). Every job output coalesces whichever ran
  (`steps.scan_hermetic.outputs.x || steps.scan_published.outputs.x`).
- Each rule job downloads the `hermetic-cli` artifact (and re-chmods the binary) when
  `cli_command` is non-empty via the shared `./.github/actions/download-hermetic-cli` composite
  action — one `uses:` line instead of the download-artifact-plus-chmod pair repeated across all
  six rule jobs (`static`, `unit-coverage`, `coverage-changed`, `mutation`, `e2e-verify`,
  `packaging` — the five static gates share the one `static` job since #410) — and runs
  `${CLI_COMMAND:-npx -y "testing-conventions${VERSION:+@$VERSION}"} <subcommand> …`. That
  `cli_command` guard is load-bearing for the `uses:` line itself, not just for whether the
  download runs: a local `uses: ./…` ref resolves against the calling job's checkout, which for
  an external consumer is *their* repo and carries no `.github/actions/` tree of ours. The guard
  is non-empty only when `caller_repository` is this repo, so wherever the ref is resolved the
  checkout already is this repo; behind any other `if:`, the same line 404s for every consumer.
  The fallback token is deliberate and load-bearing: the workflow and action `@v0` refs are resolved
  at different moments, so a consumer can transiently pair a new workflow with an old detect that
  emits no `cli_command` — the default-expansion keeps that combination running today's exact
  npx line, and it keeps the consumer execution path byte-for-byte unrouted through any new
  logic. The mutation job appends detect's pre-rendered `$TS_MUTATION_ADAPTER_ARGS` (unquoted,
  the `$EXTRA_SCOPE` pattern) because the hermetic path bypasses the npm launcher that normally
  supplies it.

Data flows through detect action outputs / `needs.detect.outputs` / step-local `env:` — never an
invented environment side-channel (AGENTS.md, "Never pass data through the environment"). The
derivation comes from `caller_repository`, never from artifact presence, so a caller that
activates hermetic mode without staging the artifact fails red at the download step — there is no
silent npx fallback in-repo. The `hermetic-wired` check pins the whole contract statically: the
guard literal, the local detect step, the `cli_command` output, the `${CLI_COMMAND:-` fallback,
and the `hermetic-cli` download in the reusable workflow; no `inputs.hermetic` and no `build-cli:`
job there; and, in each caller file, a `build-cli` job plus a `needs: [build-cli]` edge on every
`uses:` call (without the edge the build races the download and fails flaky).

The acceptance bar (#356): a PR that changes `detect`'s behavior, or a rule's, goes **red in its
own CI** before merge when that change breaks something. There is no dedicated acceptance job —
hermetic mode has no input, so every `uses:` call in the two caller workflows is the acceptance
test, exercising this branch's own `detect` and compiled CLI. Consumer-facing documentation never
mentions hermetic mode: there is nothing to document — no input exists and no job appears.

Edges: a fork PR *into* this repo runs in base-repo context, so it is gated hermetically (the
point of the gate). A fork *of* this repo carries the fork's `github.repository`, so it exercises
the published path. The direct-drive red-path self-test jobs (isolation-red, below-floor, …) drive
the CLI in their own `run:` steps rather than through a `uses:` call, so #356's caller-derivation
didn't reach them; #379 closes that by staging them off the same `hermetic-cli` artifact the
`uses:`-called jobs download — each `needs: [build-cli]` and runs `./hermetic-cli/testing-conventions`,
validating this branch's binary, not npm-latest.

## Release credentials: the publish path is OIDC-only

Every registry the release touches authenticates through GitHub OIDC. `release.yml` passes one
input and grants `id-token: write`; that permission is the entire credential surface. The publish
job mints a short-lived token per run — `rust-lang/crates-io-auth-action` for crates.io, npm's own
OIDC exchange for npm — and the PyPI upload runs in the caller's job against a PyPI trusted
publisher.

The long-lived tokens the `secrets:` block used to carry (`CARGO_REGISTRY_TOKEN`, `NPM_TOKEN`) were
bootstrap credentials, and bootstrapping is a one-time act. Trusted Publishing on both registries
binds to an **already-published** name, so the very first release of a new crate or package takes a
classic token and every release after it takes OIDC. `testing-conventions` cleared that bar long
ago: the crate has 88 versions on crates.io, and the npm package and each of its five
`@testing-conventions/*` platform packages have 90-odd, the bootstrap stub among them. putitoutthere
reads a forwarded secret as an *instruction* — its OIDC exchanges are gated on the caller's token
being empty, so passing one selects the classic-token path. The handover runs in either order:
registering the trusted publisher and dropping the secret are independent steps.

**A long-lived token is a mutable external input, which is the shape "CI hermeticity" exists to
forbid.** A registry token expires or is revoked on a wall-clock schedule that no commit records.
When it lapses, the release goes red on a commit that changed nothing about publishing, the failure
looks like whatever the registry says at that moment, and there is no commit to blame — the same
"green (or red) gate that tested the wrong thing" the hermeticity invariant rules out for required
checks. A per-run OIDC token lives exactly as long as the job that minted it.

### The worked case: the 08-20 → 08-30 release outage (#489)

Every push to `main` between 2026-08-20 and 2026-08-30 had a failed `Release` run, so nothing
published past `0.0.91`. The npm platform-build failures were fixed first; the last blocker was the
publish job's crates handler, and its cause resisted diagnosis for a specific, instructive reason:

- putitoutthere runs `cargo publish --allow-dirty --verbose` with `CARGO_TERM_VERBOSE=true` and, on
  failure, emits the captured stderr as a **single** structured log line. For this crate that stderr
  is ~312 KB (the verbose rustc invocation for each of ~180 dependencies dominates it).
- The runner writes that line only partly — 132 KB on one run, 152 KB on the next, each ending
  mid-`rustc` invocation with the JSON object unterminated. The cut is a partial write, not a fixed
  cap, so the length varies per run.
- Both cuts land around 44% of the stderr, at `oxc_regular_expression` — mid-verify-build. Reading
  the log alone, the run looks like it died compiling a dependency.

It did not. Reproducing the same command locally puts the full verify build at ~33s of a ~312 KB
stderr, ending in `Finished` and `Uploading`; the failed publish steps ran 45s and 50s, which covers
the index refresh, the packaging of 399 files, the download of 153 crates, and that whole build. So
cargo reached the upload, crates.io rejected it, and the rejection sits in the ~55% of stderr that
never reached the log. `0.0.89` is absent from crates.io, which rules out an upload that landed and
a post-upload step that failed.

Two facts stand out about what the registry rejected: the failure is deterministic (every run that
reached the publish job over ten days — 08-20, and twice on 08-30 — failed the same way, so not a
transient or a rate limit), and the crates lane was the one lane still
authenticating with a long-lived token — the npm lane, already OIDC-only, published `0.0.92` cleanly
from the same commit on a manual dispatch. Dropping the bootstrap tokens replaces the one input that
can go bad between commits, and it is the step both this file's shipping guides and putitoutthere's
own setup instructions already name as the end state.

The observability half is upstream: the truncated error line is
[putitoutthere#651](https://github.com/thekevinscott/putitoutthere/issues/651).

## Bootstrapping a new npm package name

npm trusted publishing binds to an already-published package, so the first publish of a new name
needs a classic token. `.github/workflows/bootstrap-npm.yml` is that one-shot path: it publishes a
`0.0.0-bootstrap` stub, after which the release workflow publishes real versions over it via OIDC.

1. Create an npm token and set it as the repo secret `NPM_TOKEN`. The token must bypass 2FA for
   writes, or `npm publish` fails with `EOTP: This operation requires a one-time password`. A
   classic Automation token bypasses 2FA by definition; a granular token defaults to "Require 2FA",
   so enable its "Bypass 2FA" option explicitly when creating it.
2. `gh workflow run bootstrap-npm.yml -f packages="name1,name2,..."`.
3. Once the stubs land, register Trusted Publishers on each package's npm settings page, pointing at
   this repo, `release.yml`, and the `pypi` environment.
4. Delete the `NPM_TOKEN` secret.
5. Re-trigger Release; the engine publishes real versions over the bootstrap stubs via OIDC.

The workflow can be deleted once every `testing-conventions` and `@testing-conventions/*` name
exists on the registry.

## The delegated PyPI upload

PyPI is the one registry the reusable workflow cannot publish for us. Its Trusted Publisher matching
filters candidates by `repository_owner` + `repository_name` *before* it looks at
`job_workflow_ref`, and the `repository` claim always names the caller even inside a reusable
workflow, so a token minted in putitoutthere's context never matches our TP record
([pypi/warehouse#11096](https://github.com/pypi/warehouse/issues/11096)). The upload therefore lives
in `release.yml`'s own `pypi-publish` job, and the engine *delegates* it: the `release` job builds
the distributions, hands them over as artifacts, and stops.

Delegation splits one publish into two halves that run in different workflows, and `release.yml`
owns the second half. Two rules make that half correct, and both come from the reusable workflow's
contract rather than from anything we invent:

- **Gate on `pypi_pending`, never `has_pypi`.** `has_pypi` is plan-time — the planned matrix held
  pypi rows — which is also true for a run that merely *rebuilt* wheels. `packages/rust/**` sits in
  the Python package's `globs`, so every Rust change cascades a Python rebuild, and the engine can
  then decide the Python version is already live and hand over nothing. `pypi_pending` is
  publish-time: `'true'` only when the engine reached the package and delegated its upload, `'false'`
  when the version is already on PyPI. Pair it with `!cancelled()` so a red npm or crates.io lane
  does not swallow a PyPI upload the engine already handed over — the three registries are
  independent.
- **Tag after the upload, from registry truth.** The engine tags crates.io and npm as it publishes
  them, and deliberately leaves a delegated PyPI package untagged: a tag records what shipped, and
  at hand-over nothing has. The `pypi-tag` job cuts it once the upload lands, reading the version
  PyPI reports as live, which makes it idempotent and self-healing.

The two rules hold each other up. A published version that goes untagged is one the next run's plan
reads as still owed, so it recomputes the same version and delegates it a second time — and a
`pypi-publish` job gated on `has_pypi` runs anyway and uploads files PyPI already stores, which is a
`400 File already exists` and a red `Release`. The worked case: three interleaved red runs over
2026-08-30/31 rejected `0.0.92`, `0.0.93` and `0.0.94` in turn, each one re-uploading a version a
green run minutes earlier had already published, while `testing-conventions-py-v0.0.93` and
`-v0.0.94` ended up naming commits whose builds never reached PyPI at all.

This wiring is a **contract with `@v0`, and contracts drift.** Both rules arrived upstream together
in [putitoutthere#623](https://github.com/thekevinscott/putitoutthere/issues/623); `release.yml` kept
the older shape for ten days and went red on the difference. When `@v0` moves, re-read
putitoutthere's README → "Publishing to PyPI" and match the template it publishes there. No
`tc-checks` wiring gate guards this: a regression is loud — `Release` fails on the upload — and
wiring gates are earned by silent, correctness-affecting failures, not by plumbing that announces
itself (see AGENTS.md, "Wiring gates are earned").

## Rolling release: how `@v0` advances

`@v0` is a **moving major tag**: consumers pin `…/testing-conventions.yml@v0` and `…/actions/detect@v0`, and the tag is force-moved forward on each release so every consumer tracks `main`. We own all consumers and fix forward — this is rolling release, the opposite of a semver pin.

The tag is advanced by a dedicated workflow, `.github/workflows/move-major-tag.yml`, **not** inline in `release.yml`. It is **gated on a successful publish**: it triggers via `workflow_run` on the `Release` workflow completing and runs only when `conclusion == 'success'` (on `main`). That gate is the one place this repo departs from the generic "move the tag on every push to `main`" recipe, and it is non-negotiable:

The reusable workflow runs the **published** binary (`npx testing-conventions` → latest on npm), but the workflow *file* is frozen at `@v0`. If `@v0` advanced to a commit whose workflow invokes a subcommand the npm-latest binary doesn't expose yet (a rename/addition — the #55 class of break), every consumer running in the publish window would get new-workflow + old-binary → `unrecognized subcommand`. Publishing the binary is this repo's analog of committing a built `dist/`: ship the runtime first, then move the tag. `needs: release` (#92) did this inline; `move-major-tag.yml` does it as a named, single-responsibility workflow.

Two safety properties:

- **Concurrency** (`group: move-major-tag`, `cancel-in-progress: true`): the newest release wins; a stale in-flight move is cancelled.
- **Forward-only**: the tag moves to the released SHA only when that SHA is a descendant of the current `@v0` (otherwise it's a no-op), so out-of-order release runs can never rewind `@v0`. It also bootstraps `@v0` on first run.

The forward-only logic is a repo-only, pytest-covered helper — `internals/move-major-tag/src/move_major_tag.py`, behind a small git boundary so it carries integration tests (git mocked) and e2e tests (a real repo with a local remote), run by `move-major-tag-tests.yml` — exactly like the `detect` helper. The workflow YAML only wires the trigger, the checkout, and the env; it holds no logic.

The wiring is guarded in CI (`rolling-release-wired` in `testing-conventions-selftest.yml`): a regression that re-introduces an inline or un-gated tag move fails the self-test.

### Validated promotion: verify before `@v0` advances (#357)

Publish-gating is necessary but not sufficient. It proves the binary published; it does **not** prove that the combination the tag move is about to bless — the *new* workflow file, the *published* binary, the *current* `@v0` detect — is green over the consumer surface. A release can publish a perfectly good binary and still move `@v0` into a combination that fails the self-test/dogfood suite (the packaging case is the worked example): a red `main` with no commit to point at, and every consumer red on their next run. Layer 1 (#356) closes this for the *merge* gate — every PR is gated on `(HEAD workflow × HEAD detect × HEAD-built binary)` — but the promotion itself was still an unguarded deploy. #357 gates it: between publish and tag-move, run the full self-test + dogfood surface **pinned to the just-published immutable version**, and advance `@v0` **only if green**. Fail **closed** — any red leaves `@v0` exactly where it was, so `main` and consumers stay on the last-good release.

**The verification is the published path, forced by the existing seam.** Calling the reusable workflow with `version: <just-published>` is, by #356's derivation, exactly what selects the published path: the caller *is* this repo, but `version != ''`, so `hermetic()` is false and every rule job runs the real `npx testing-conventions@<version>` — the consumer ergonomic, not the hermetic build-from-HEAD. No new mechanism; the `version` input the seam was designed for is the whole lever. The just-published version is resolved from the `testing-conventions-npm-v*` tags reachable from the release commit (putitoutthere tags on publish), so it is pinned to the release, not read from `npx`-latest at some later wall-clock moment.

**"Verify at the release, not at detect-pinned-to-the-release" is structurally forced, not a smaller option we chose.** The thing a consumer runs the instant `@v0` moves is the workflow file whose `detect` step literally reads `…/actions/detect@v0`. A "more complete" verification that re-pinned `detect` to the release commit would assemble a *different file* than the one being promoted — verifying a workflow no consumer ever executes, which is the precise "green gate that tested the wrong thing" this epic exists to kill. And it is not merely undesirable but **unconstructable**: `uses:` refs cannot be dynamic, so the combination `(new workflow with its literal @v0 × detect resolved at the new tag)` does not exist until the tag moves — the ref target isn't there yet. This is the same shape as #353's original argument for moving consumer-surface testing from pre-merge to pre-promotion: there, the *artifact* didn't exist yet; here, the *ref target* doesn't. The logic that a detect-pinned verification would have covered — the new-workflow × new-detect combination, the #351 incident class — is already proven before merge by Layer 1's hermetic gate, which runs HEAD's detect against HEAD's workflow on every PR. So the coverage isn't dropped; it's supplied where it *can* be constructed.

**The one named residual, and its cover.** `detect` has no publish step — its "publish" *is* the tag move — so its provenance risk is not a bad binary but the fetch/layout mechanics at the promoted commit. GitHub resolves a remote composite action (`owner/repo/.github/actions/detect@v0`) by fetching the repo at that ref, and `detect`'s `action.yml` reaches its implementation via `$GITHUB_ACTION_PATH/../../../internals/detect/src/detect.py`. Layer 1 never exercises that remote-fetch path (it uses the workspace-local `./.github/actions/detect`), and the version-pinned verification exercises it only at the *old* tag. What slips through both: a file move that breaks that relative path, or an `export-ignore`/archive quirk that strips `internals/` from the fetched action — green in every gate, then every consumer's `detect` job dies the moment `@v0` moves. The cover is one colocated-tested layout check in the verification, before the tag move: `git archive <release-sha>` and assert `internals/detect/src/detect.py` (and, generally, the relative-path targets `action.yml` reaches) are present in the archive. That literally simulates the action-fetch mechanism against the exact commit being promoted, closing the realistic remainder without touching the workflow's derivation at all.

**Two execution invariants:**

- **"Narrow" scopes `detect`, never the suite.** Verification is still the *full* self-test + dogfood surface — every fixture, every rule job — just run version-pinned rather than hermetic. Narrowing means only that `detect` stays at the current `@v0` (per the unconstructable argument above), not that fewer jobs run.
- **Pin to the release SHA, not "current `main`."** A `workflow_run`-triggered verification checks out whatever the default branch is at trigger time; a commit landing between publish and verification would have it verify a workflow file that is *not* the one the tag will bless. Local `uses:`/`./` reusable-workflow calls resolve at the verify workflow's own commit and their refs cannot be an expression, so the mechanism that pins both the workflow file *and* its inner `uses:` to an arbitrary commit is a `workflow_dispatch` targeting the release commit. `workflow_dispatch` takes a branch or tag ref, never a bare SHA, so verification creates a **throwaway tag at the release SHA** (`verify-release-<sha>`, cleaned up in a `finally`; no workflow triggers on `push: tags:`, so creating it fires nothing), dispatches the self-test and dogfood workflows at that tag with `version: <just-published>`, and polls their conclusions — pinning each dispatched run to the exact release commit, the same forward-only discipline `move-major-tag` applies to `@v0` itself.

**Mechanism.** The direct `Release`-success → `move-major-tag` chain becomes `Release`-success → **verify-and-promote**. On a successful publish, verify-and-promote: (1) resolves the release SHA and the just-published npm version from the tags reachable there; (2) runs the layout check against the release SHA; (3) dispatches `testing-conventions-selftest.yml` and `dogfood.yml` at a throwaway tag on the release SHA with the pinned `version`, and polls until both conclude; (4) advances `@v0` via the unchanged forward-only `move_major_tag.py` **only** when the layout check and both dispatched runs are green. Every non-trivial step is the colocated-tested `tc-checks verify-release` command (`internals/checks`, `checks/utils/verify_release.py` behind an injected git/`gh` boundary — the `build-hermetic-cli` pattern, so the genuinely-equivalent boundary/timing mutants carry reasoned `testing-conventions.toml` exemptions rather than living where none is possible); the workflow YAML wires triggers, checkouts, and env, and holds no logic. It lives in `internals/checks` because its `gh` boundary can't be exercised for real in CI, so a handful of mutants need the reasoned exemptions only a package's `testing-conventions.toml` can grant. The `rolling-release-wired`/`verify-release-wired` static checks guard that the tag move stays gated on verification, so a regression that re-introduces a bare publish-only promotion fails the self-test.

## The move-major-tag helper's package (`internals/move-major-tag`)

`move_major_tag.py` (the forward-only `@v0` tag-advance helper, #235) lives in its own uv package, `internals/move-major-tag` (#452), mirroring `internals/detect`: `src/move_major_tag.py` with its colocated `move_major_tag_test.py`, integration tests (the git boundary mocked) and e2e tests (a real repo with a local remote) under `tests/`, and pytest a dev-dependency pinned in the package's `uv.lock`. `move-major-tag.yml` invokes it as a plain stdlib script (`python3 internals/move-major-tag/src/move_major_tag.py`, no install step); `move-major-tag-tests.yml` runs the three-tier suite from the package's own lock. Like `internals/detect` (see below), its colocated unit test alone sits below the coverage floor, so its gate is that dedicated pytest workflow rather than a `dogfood.yml` call.

It was the last loose script under `.github/scripts/`, held to the conventions by `dogfood-github-helpers.yml` — a job that ran the published binary via `npx`, the n-1 skew class (#206, #351, #355) the hermetic gates exist to close. The migration emptied `.github/scripts/`, so that workflow and its `github-helpers-wired` selftest guard (#329) retired with it: no code lives under `.github/`, and no required check outside the reusable workflow's consumer path invokes an unpinned `npx testing-conventions`.

## The self-test checks package (`internals/checks`)

The #302 wiring/assertion checks are consolidated into a single uv package at `internals/checks/` — `pyproject.toml` + `uv.lock` + a `src/checks/` layout (epic #321, complete). `checks/cli.py` is a `@click.group()` (`tc-checks`) that composes each check as a subcommand; each check lives in its own subpackage — `checks/<check>/cli.py` holds a pure predicate (or, for the failure-path group, a hardcoded `CHECKS` list) and a `@click.command()`, with a colocated `cli_test.py`. Shared code lives in `checks/utils/`: `check_failed.py` (the `CheckFailed` `click.ClickException` that prints a `::error::` annotation), `run_checks.py` + `failure_reason.py` (the failure-path orchestrator and its exit-code decision), and `job_block.py` (isolating a named job's YAML region). A self-test job runs `uv run --project internals/checks tc-checks <check>`.

The layout mirrors `packages/python`, whose importable package sits in `packages/python/python` while `packages/python/tests` holds the integration/e2e suite: `source` for the dogfood points at the **inner** `internals/checks/src`, not the package root, so the static gates recurse only the source tree. The colocated `cli_test.py` units drive each check's `@click.command` through its `.callback` (no `CliRunner`, which is a third-party collaborator the isolation lint flags) and import only the unit under test — so the colocated suite alone reaches the 100% coverage floor. The full e2e suite (`CliRunner` over the real workflow file) lives at `internals/checks/tests/e2e`, a sibling **outside** the scanned `src/`; a `*_test.py` e2e file *inside* the scan would be read as an un-isolated unit test and red the lint. The package root (`internals/checks`, where the `pyproject.toml` lives) is still derived for the coverage/mutation venv.

The packaging gate's `packaging_build` derivation covers `internals/checks` too (a plain `uv build`, #335), so the dogfood packaging job builds this package's own distributions and scans them — and both must exclude the colocated `*_test.py` units the same way any other zero-config Python package would, or the scan rejects the artifact as shipping its tests (#354). `uv build` produces a wheel *and* an sdist, and hatchling's `[tool.hatch.build.targets.wheel]` / `[tool.hatch.build.targets.sdist]` exclude independently of each other — an exclude scoped to only the wheel target leaves the sdist (`.tar.gz`) shipping every test file untouched. The top-level `[tool.hatch.build] exclude = ["**/*_test.py"]` applies to both targets at once. Tests still run from the source tree (`.venv`/`uv run pytest`), never from a built artifact, so the exclude has no effect on execution — only on what `uv build` packages.

It lives under `internals/` with the repo's other first-party helper packages. As a real package it is dogfooded through the **shipped reusable workflow** (`dogfood.yml`, `path: internals/checks/src`) — colocated-test, isolation, coverage, integration-lint, and diff-scoped mutation — exactly like `packages/python`.

## The detect action's package (`internals/detect`)

`detect.py` (the `detect` composite action's implementation, #189/#277 onward) moved out of `.github/actions/detect/` into its own uv package, `internals/detect/` (#363), mirroring `internals/checks`: custom logic earns a real package with real test tiers, not a loose script under `.github/`. `internals/detect/src/detect.py` is a single top-level module (no subpackage — one file, no CLI subcommands to compose), with its colocated `detect_test.py` beside it and the integration/e2e suites at `internals/detect/tests/`, a sibling outside `src/`, exactly like `internals/checks`.

Unlike `internals/checks`, it is **not** dogfooded through the shipped reusable workflow. `internals/checks`' colocated `cli_test.py` units alone reach the coverage floor, so scoping the dogfood job's `source` to `src/` (excluding the e2e suite entirely) works cleanly. `detect.py`'s colocated `detect_test.py` alone does not — `compute_outputs`'s orchestration is exercised only by the integration suite (filesystem mocked) and the full script only by the e2e suite. Scoping to `internals/detect/src` alone therefore fails the coverage floor (the integration/e2e suites are outside the scan and never run); scoping to the package root instead (`internals/detect`, so all three tiers run together) fails `unit lint`'s `unmocked-collaborator` rule, because that rule has no concept of test tiers — once a first-party package is declared (any `pyproject.toml`), it flags *every* `*_test.py` under the scanned root that imports the package unmocked, `detect_integration_test.py` and `detect_e2e_test.py` included. (This also explains why `detect.py` silently passed `dogfood-github-helpers.yml`'s isolation check for years despite the same nested layout: `.github/actions` never had a `pyproject.toml`, so the rule's first-party-package lookup found nothing and reported no violations at all — not because the layout satisfied it.) `detect.py` keeps its existing, proven test-quality gate instead: `detect-action.yml`'s dedicated pytest run across all three tiers together (100% coverage via plain `coverage.py`), independently of this tool's own gates.

`.github/actions/detect/action.yml` is unaffected by the move — it is a thin composite-action manifest, not Python, and it is the file every consumer's `uses: …/actions/detect@v0` reference resolves against. Its `run:` step now points at `$GITHUB_ACTION_PATH/../../../internals/detect/src/detect.py`: GitHub Actions checks out the *whole* repo at the pinned ref to resolve a composite action (not just the action's own subdirectory), so a relative path climbing back out to the repo root and down into `internals/` resolves identically whether the action is used locally (`./.github/actions/detect`) or externally (`owner/repo/.github/actions/detect@ref`). The `uses:` contract itself never changes, so this is not a breaking change for any consumer and needs no `MIGRATIONS.md` entry.

**The `outputs:` block is the forwarding contract.** A composite action forwards exactly the outputs its manifest declares. `steps.detect.outputs.<name>` resolves to the empty string for any name the block omits, however faithfully `detect.py` wrote that name to `GITHUB_OUTPUT`. The manifest and the script therefore hold one contract between them: every key `compute_outputs` returns has an `outputs:` entry, and each entry forwards `${{ steps.scan.outputs.<the same name> }}`. `internals/detect`'s e2e suite asserts both halves against the real `action.yml` — as set equality, so the next output is covered on the day it is added rather than by a fresh one-off assertion.

`static_languages` is the case that earned the check. The manifest never declared it, so the `static` job's `fromJSON(needs.detect.outputs.static_languages || needs.detect.outputs.integration_lint_languages)` matrix took the fallback arm on every run. The two sets compute identically today, so the job fanned out over the right languages for the wrong reason and stayed green. The signal would have arrived only as a silently wrong matrix, on the first release where a language joined one set and not the other.

### The scan's invocation is an external contract (willfire)

The fleet gate names a consumer PR's checks before the run: pr-monitor compares the checks that reported against the set the commit's own logic schedules, and the reusable workflow's per-language matrices (`fromJSON(needs.detect.outputs.*)`) make that set a function of the scan's outputs. To compute them, [willfire#19](https://github.com/thekevinscott/willfire/issues/19) **executes the scan itself**: it fetches this repo at the SHA `@v0` resolves to and runs the script the way the composite action's `run:` step does, against a checkout of the consumer's PR head. One implementation, two call sites — the prediction and the run stay in agreement because both execute the same code against the same commit.

The invocation therefore has an external caller, and its shape is a **stable contract**:

- **Interpreter.** Stdlib-only `python3` (3.11+, for `tomllib`), invoked directly — `python3 internals/detect/src/detect.py` — with the working directory at the consumer checkout root.
- **Inputs.** The five env vars the action manifest binds — `LANGUAGES`, `SCAN_PATH`, `CONFIG`, `CALLER_REPOSITORY`, `VERSION` — carrying the caller's `with:` literals; their meanings live in `action.yml` and the module docstring. For an external caller `CALLER_REPOSITORY` is the consumer's repo, so the scan takes the published path — the same branch the consumer's own run takes.
- **Outputs.** Lines appended to the file named by `GITHUB_OUTPUT`: `name=value` for a single-line value, the runner's heredoc form (`name<<DELIM`) for a multi-line one. The stdout summary line is for humans, outside the contract.

A change to any of these — the script's path, the interpreter floor, an env name or meaning, the output encoding — breaks the external evaluator. Make it deliberately, and update willfire's recipe in the same motion.

### Our own PRs need the same resolution, granted explicitly (#502)

The paragraph above is the *consumer* path, where willfire resolves the matrix by executing the scan at the SHA `@v0` names. `dogfood.yml` calls the reusable workflow directly, so the same matrices are unresolvable on this repo's own PRs — and pr-monitor executes nothing unless told to. `.github/workflows/pr-monitor.yml` grants it:

```yaml
execute: thekevinscott/testing-conventions:detect
```

The grant names the repo the workflow *file* lives in — for a reusable workflow, the callee — and the job that computes the matrix. Without it the gate fails closed on `Unresolvable check names`, which is the correct behavior rather than a defect: a predicted set missing a leg cannot be compared against the observed one, because a missing name is indistinguishable from a leg that was never predicted. Willfire 0.1.31 removed per-job grants, so the grant's scope is the whole prediction: every job the prediction reaches may run for real, in a docker sandbox, at the predicted commit. The name `detect` is still parsed, and still records in the log what the grant meant — the action prints `naming ["detect"] no longer restricts which jobs run`. The grant is therefore a statement of trust in that entire surface, which here is `dogfood.yml`'s own fan-out: the stdlib-only `detect` scan and the reusable workflow's gate jobs, which dogfood runs on every PR anyway.

The action tracks the `v1` tag, so the gate runs willfire's current release. `v1` names willfire `0.1.31`, which binds a root workflow's unsupplied `inputs` to the empty string — the reusable workflow's `scan_hermetic` guard (`inputs.version == ''`) decides, and the prediction names every check.

With epic #321 complete, every #302 wiring/assertion and failure-path check lives in `internals/checks` as a `tc-checks <check>` subcommand; the flat `.github/scripts/<check>/` dirs are gone, and each self-test job invokes `uv run --project internals/checks tc-checks <check>` after `astral-sh/setup-uv`. The full inventory, by original sub-issue:

- **Wiring assertions (#323):** `mutation-wired`, `isolation-wired`, `coverage-rust-wired`, `colocated-rust-wired`, `diff-scoped-wired`, `e2e-verify-wired`, `e2e-verify-checks-out-pr-head` (block-scoped to the `e2e-verify` job, replacing the old `awk` range), `e2e-verify-scope-wired`, `rolling-release-wired` (two selftest steps folded into one command over two file arguments).
- **Detect wiring (#324):** `wiring-detect-action`, `wiring-packaging-default-on`, `wiring-e2e-default-on`, and `detect-routes-python` — the last keeps its `uses: ./.github/actions/detect` step in the job and passes the action's `isolation_languages` output as a single-quoted JSON CLI argument.
- **Feature-input wiring (#325):** `build-command-wired`, `gates-wired`, `rust-toolchain-wired`.
- **Package-root wiring (#326):** `coverage-package-root-wired`, `packaging-package-root-wired`, `mutation-package-root-wired` — each isolates a job's YAML region and asserts it references `needs.detect.outputs.package_root`.
- **Detect-output validations (#327):** `detect-package-root-ts`, `detect-package-root-py` — each runs `./.github/actions/detect` against a monorepo fixture and hands the outputs to a pure `evaluate` returning the first mismatch's message.
- **Failure-path (#328):** `isolation-red`, `below-floor`, `mutation-gate`, `python-mutation-clean`, `packaging-red`, `coverage-rust-red`, `integration-lint-new-arms-trip`, `packaging-package-root-red`, `colocated-rust-red` (#379) — each runs hermetic-CLI (`./hermetic-cli/testing-conventions`, from `config.HERMETIC_CLI`) invocations from a `CHECKS` list and asserts the exit code via `failure_reason`.
- **github-helpers-wired (#329):** retired in #452 along with `dogfood-github-helpers.yml`, the workflow whose arms it pinned — `move_major_tag.py`, the last loose script it dogfooded, is a real package now (see "The move-major-tag helper's package").
- **red-path-hermetic-wired (#379):** asserts every failure-path job downloads the `hermetic-cli` artifact (`needs: [build-cli]` + `./.github/actions/download-hermetic-cli`), so none drives npm-latest.

The static checks hold their inspection in a pure predicate over the workflow file; the failure-path group holds a `CHECKS` list run through the shared `run_checks` orchestrator. Either way the colocated `cli_test.py` drives the pure logic in isolation, the `@click.command()` raises `CheckFailed` (a `::error::` annotation) on a failure, and a sibling `CliRunner` e2e suite exercises the real boundary — held to the same coverage and mutation bar as any shipped source.

The two pre-existing first-party helpers were resolved per the #321 open question: `detect.py` moved to `internals/detect` (#363), and `move_major_tag.py` — which stayed a loose script under `.github/scripts/` until #452 — moved to `internals/move-major-tag`, emptying `.github/scripts/` entirely.

## Rust CI: nextest, and why the coverage job's cache needed no change (#370)

`rust.yml`'s `integration` job ("Integration + e2e tests + coverage (95%)") runs the ~65 files under `packages/rust/tests/` through `cargo llvm-cov`. #370 (epic #366) asked for two things: a reliable, distinct cache for the coverage-instrumented build, and running under `nextest`. Only the second turned out to be real.

**The cache ask was already satisfied.** `Swatinem/rust-cache@v2` bakes the GitHub Actions job name into its default key, so `lint`, `unit`, `integration`, and `build` already get four separate, non-colliding caches — confirmed by inspecting live cache-key strings in CI logs (`v0-rust-integration-Linux-x64-…` vs `v0-rust-lint-Linux-x64-…`). This separation isn't incidental: `cargo llvm-cov` compiles under `-C instrument-coverage` into a distinct `target/llvm-cov-target/` directory, so its build artifacts could never usefully share a cache with the other jobs' plain `cargo build`/`cargo test` output regardless of key tuning. The actual (occasional) cache misses trace to `dtolnay/rust-toolchain@stable` being unpinned — a rustc point-release bump invalidates all four jobs' caches simultaneously — but pinning it was out of scope here: the same action is used seven more times in the *shipped* reusable workflow (`testing-conventions.yml`), and pinning there is a consumer-facing toolchain-provisioning decision with its own maintenance cost, not an internal CI tweak. Cold-vs-warm compile time for this job also turned out to be a modest 15–30% gap in practice, not the dominant cost — so no cache changes were made.

**`nextest` is the real fix.** The 65 integration-test files each compile to their own binary; the default harness runs them one at a time, and several cost multiple seconds to tens of seconds because they shell out to real subprocesses (pytest, `npx vitest`, `cargo-mutants`) — that serial cost, not compilation, is what dominates the job's wall clock. `cargo llvm-cov nextest --ignore-filename-regex 'main\.rs' --fail-under-lines 95` is a direct drop-in for the previous `cargo llvm-cov --ignore-filename-regex 'main\.rs' --fail-under-lines 95`: `--fail-under-lines`/`--ignore-filename-regex` are `cargo-llvm-cov`'s own report-gating flags, applied identically regardless of which test-runner subcommand executes the tests. The crate has zero doctests, so nextest's well-known "doesn't run doctests" gap costs nothing here.

One correctness question was worth answering empirically before landing this, not assuming: `mutation.rs`'s `ensure_cargo_mutants()` provisions a shared, version-scoped binary cache (`~/.cache/testing-conventions/cargo-mutants-<version>`) with no file locking — a bare "does the binary exist, if not run `cargo install`" check. nextest runs each test *binary* in its own OS process, in parallel, which could mean several processes racing to provision that shared cache simultaneously on a cold cache. Verified locally: cleared the cache, ran the mutation-Rust tests concurrently — every run landed on an intact, correctly-sized binary, no corruption. **That verification checked the race's safety but not its cost, and cost was the actual gap**: PR #383's own first CI run hit an evicted provisioning cache and took 6m59s instead of the expected ~2m — not nextest overhead, but four concurrent full from-source `cargo install cargo-mutants` compiles racing on a 4-vCPU runner (the old harness's one-binary-at-a-time model meant a cold cache only ever paid one serial install; nextest's cross-binary parallelism was the first thing to run several cargo-mutants-driving tests concurrently for real). Fixed in #385 with an advisory file lock around the install, re-checking for the binary after acquiring it — concurrent callers now wait for one install instead of each duplicating it, restoring the old cold-cache cost profile regardless of test-runner concurrency.

With that fixed, the real warm-cache comparison holds up: a subsequent PR's run (provisioning cache hit) measured the `cargo llvm-cov nextest` step at **2m08s**, against the pre-nextest baseline's 2m23s–2m54s — a modest, real win, not the dramatic cut the epic's original profiling hoped for (that number was dominated by `dogfood-github-helpers.yml`'s Python mutation step, addressed separately, and by the since-resolved #364 packaging-fixture bug).

## Python CI: build the wheel once (#371)

`python.yml`'s `build` job used to run `maturin build --release` across the full `3.9`–`3.13`
matrix, and `plugin` built it again for its own `3.9`/`3.13` matrix — seven Rust compiles of the
same crate per PR run. The package is maturin `bindings = "bin"`
(`packages/python/pyproject.toml`): the wheel ships the Rust binary with no per-Python-version
native extension, so every matrix leg was compiling and wrapping the identical artifact.

Verified before implementing, per the issue's own caveat that this could collapse to a no-op if
the wheel tag turned out to be version-specific: `maturin build --release` produces
`testing_conventions-<version>-py3-none-manylinux_*.whl` — the `py3-none-` tag is Python's
own marker for "any CPython 3.x on this platform," confirmed by installing and running the same
`.whl` under real 3.10–3.13 venvs locally (3.9 wasn't available to test directly, but the tag
guarantees it). One wheel is correct for the whole matrix.

`build-wheel` now builds it once and uploads it as an artifact (`actions/upload-artifact@v7`,
matching the naming/versioning this repo already uses for the reusable workflow's own
`packaging_artifact` wiring); `build` and `plugin` both `needs: build-wheel` and
`actions/download-artifact@v8` the same wheel instead of rebuilding, matrixing only the cheap
consumer-facing check each was actually testing — `pip install` + `--version` for `build`,
`pip install` + the plugin's pytest suite for `plugin`. Neither downstream job needs the Rust
toolchain, `rust-cache`, or `maturin` installed anymore; they only need Python and the
already-built wheel. `plugin` still checks out the repo (unlike `build`) because it runs
`pytest tests/` against the checked-out integration-test files, which aren't part of the wheel.

## Node CI: cache the pnpm store (#372)

`node.yml`'s four jobs (`lint`, `typecheck`, `test`, `build`) each ran `pnpm install
--no-frozen-lockfile` from a cold store — the same dependency set fetched and linked four times
per PR. Each job now sets `cache: pnpm` on its `actions/setup-node@v6` step, so the store
restores from a hashed key instead of re-downloading every run. `pnpm/action-setup@v5` already
ran before `setup-node` in every job (needed regardless, to put `pnpm` on `PATH` before `pnpm
install`) — that ordering is also what `cache: pnpm` needs, since `setup-node` shells out to
`pnpm store path` to resolve what to cache, so no step reordering was required.

**`cache-dependency-path` points at `package.json`, not `pnpm-lock.yaml`.** First attempt used
the lockfile (the obvious hash input, and what the action's docs lead with) and it broke CI:
`.gitignore` has a blanket `pnpm-lock.yaml` rule — **no pnpm lockfile is committed anywhere in
this repo** — so there was nothing in the checkout for `cache-dependency-path` to hash
("Some specified paths were not resolved, unable to cache dependencies"). `package.json` is the
closest committed proxy for "did the intended dependency set change." This also retroactively
answers the issue's other question — whether `--no-frozen-lockfile` is deliberate: it has to be,
since `--frozen-lockfile` requires a lockfile to freeze against, and none is ever committed.
Left untouched, now with a real reason on record rather than an absence of one.
