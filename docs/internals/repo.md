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
  - `internals/checks` and the jobs with no uv project of their own (`rust.yml` integration, `dogfood-github-helpers.yml`, `move-major-tag-tests.yml`) — `coverage`/`pytest`/`cosmic-ray` still float via `uv run --with … --no-project`; #446 pins them in `internals/checks`'s lock and has those jobs borrow its environment via `--project internals/checks`.
  - the `packages/python` wheel build (`python.yml`) — `maturin` still floats via `--with maturin`; #448 pins it in `[build-system].requires`, where a PEP 517 build backend is declared.

- **The adapter is this repo's code, so it resolves from source, not the wheel.** `unit mutation --language python` spawns `python3 -m testing_conventions.mutation.main` (#248). Installing the published `testing-conventions` wheel to supply that module — `pip install testing-conventions`, or `uv run --with testing-conventions` — runs the *last release's* adapter over this PR's fixtures, the same `@v0`/npm version-skew class the hermetic merge gate (#356) closes for the CLI binary. The `hermetic-cli` artifact stages the Rust binary and the node `dist/`, not the Python adapter, so it does not reach this. The fix, which `rust.yml`'s integration job already models, is `PYTHONPATH: ${{ github.workspace }}/packages/python/python` — the adapter's source tree, ahead of any installed copy — so the adapter under test is the PR's. `cosmic-ray` (its engine, otherwise pulled in transitively by the wheel) is then layered explicitly via `--with cosmic-ray`.

The consumer-path jobs are the deliberate exception, and only for the CLI binary they validate: `dogfood-github-helpers.yml` still runs `npx testing-conventions` (the ergonomic a consumer gets), and `python.yml` still `pip install`s the built `dist/*.whl` to load the pytest plugin the way an installed consumer does. Even there the *adapter* the mutation arm spawns resolves from source via `PYTHONPATH`, because its version skew is a separate defect the published-binary exception was never meant to cover. The direct-drive red-path self-test jobs route through `hermetic-cli` rather than ad-hoc `npx` (#379), and the reusable workflow's own Python provisioning is uv-only (#399).

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
  `packaging` — the four static gates share the one `static` job since #410) — and runs
  `${CLI_COMMAND:-npx -y "testing-conventions${VERSION:+@$VERSION}"} <subcommand> …`. The
  fallback token is deliberate and load-bearing: the workflow and action `@v0` refs are resolved
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

## Rolling release: how `@v0` advances

`@v0` is a **moving major tag**: consumers pin `…/testing-conventions.yml@v0` and `…/actions/detect@v0`, and the tag is force-moved forward on each release so every consumer tracks `main`. We own all consumers and fix forward — this is rolling release, the opposite of a semver pin.

The tag is advanced by a dedicated workflow, `.github/workflows/move-major-tag.yml`, **not** inline in `release.yml`. It is **gated on a successful publish**: it triggers via `workflow_run` on the `Release` workflow completing and runs only when `conclusion == 'success'` (on `main`). That gate is the one place this repo departs from the generic "move the tag on every push to `main`" recipe, and it is non-negotiable:

The reusable workflow runs the **published** binary (`npx testing-conventions` → latest on npm), but the workflow *file* is frozen at `@v0`. If `@v0` advanced to a commit whose workflow invokes a subcommand the npm-latest binary doesn't expose yet (a rename/addition — the #55 class of break), every consumer running in the publish window would get new-workflow + old-binary → `unrecognized subcommand`. Publishing the binary is this repo's analog of committing a built `dist/`: ship the runtime first, then move the tag. `needs: release` (#92) did this inline; `move-major-tag.yml` does it as a named, single-responsibility workflow.

Two safety properties:

- **Concurrency** (`group: move-major-tag`, `cancel-in-progress: true`): the newest release wins; a stale in-flight move is cancelled.
- **Forward-only**: the tag moves to the released SHA only when that SHA is a descendant of the current `@v0` (otherwise it's a no-op), so out-of-order release runs can never rewind `@v0`. It also bootstraps `@v0` on first run.

The forward-only logic is a repo-only, pytest-covered helper — `.github/scripts/move-major-tag/move_major_tag.py`, behind a small git boundary so it carries integration tests (git mocked) and e2e tests (a real repo with a local remote), run by `move-major-tag-tests.yml` — exactly like the `detect` helper. The workflow YAML only wires the trigger, the checkout, and the env; it holds no logic.

The wiring is guarded in CI (`rolling-release-wired` in `testing-conventions-selftest.yml`): a regression that re-introduces an inline or un-gated tag move fails the self-test.

### Validated promotion: verify before `@v0` advances (#357)

Publish-gating is necessary but not sufficient. It proves the binary published; it does **not** prove that the combination the tag move is about to bless — the *new* workflow file, the *published* binary, the *current* `@v0` detect — is green over the consumer surface. A release can publish a perfectly good binary and still move `@v0` into a combination that fails the self-test/dogfood suite (the packaging case is the worked example): a red `main` with no commit to point at, and every consumer red on their next run. Layer 1 (#356) closes this for the *merge* gate — every PR is gated on `(HEAD workflow × HEAD detect × HEAD-built binary)` — but the promotion itself was still an unguarded deploy. #357 gates it: between publish and tag-move, run the full self-test + dogfood surface **pinned to the just-published immutable version**, and advance `@v0` **only if green**. Fail **closed** — any red leaves `@v0` exactly where it was, so `main` and consumers stay on the last-good release.

**The verification is the published path, forced by the existing seam.** Calling the reusable workflow with `version: <just-published>` is, by #356's derivation, exactly what selects the published path: the caller *is* this repo, but `version != ''`, so `hermetic()` is false and every rule job runs the real `npx testing-conventions@<version>` — the consumer ergonomic, not the hermetic build-from-HEAD. No new mechanism; the `version` input the seam was designed for is the whole lever. The just-published version is resolved from the `testing-conventions-npm-v*` tags reachable from the release commit (putitoutthere tags on publish), so it is pinned to the release, not read from `npx`-latest at some later wall-clock moment.

**"Verify at the release, not at detect-pinned-to-the-release" is structurally forced, not a smaller option we chose.** The thing a consumer runs the instant `@v0` moves is the workflow file whose `detect` step literally reads `…/actions/detect@v0`. A "more complete" verification that re-pinned `detect` to the release commit would assemble a *different file* than the one being promoted — verifying a workflow no consumer ever executes, which is the precise "green gate that tested the wrong thing" this epic exists to kill. And it is not merely undesirable but **unconstructable**: `uses:` refs cannot be dynamic, so the combination `(new workflow with its literal @v0 × detect resolved at the new tag)` does not exist until the tag moves — the ref target isn't there yet. This is the same shape as #353's original argument for moving consumer-surface testing from pre-merge to pre-promotion: there, the *artifact* didn't exist yet; here, the *ref target* doesn't. The logic that a detect-pinned verification would have covered — the new-workflow × new-detect combination, the #351 incident class — is already proven before merge by Layer 1's hermetic gate, which runs HEAD's detect against HEAD's workflow on every PR. So the coverage isn't dropped; it's supplied where it *can* be constructed.

**The one named residual, and its cover.** `detect` has no publish step — its "publish" *is* the tag move — so its provenance risk is not a bad binary but the fetch/layout mechanics at the promoted commit. GitHub resolves a remote composite action (`owner/repo/.github/actions/detect@v0`) by fetching the repo at that ref, and `detect`'s `action.yml` reaches its implementation via `$GITHUB_ACTION_PATH/../../../internals/detect/src/detect.py`. Layer 1 never exercises that remote-fetch path (it uses the workspace-local `./.github/actions/detect`), and the version-pinned verification exercises it only at the *old* tag. What slips through both: a file move that breaks that relative path, or an `export-ignore`/archive quirk that strips `internals/` from the fetched action — green in every gate, then every consumer's `detect` job dies the moment `@v0` moves. The cover is one colocated-tested layout check in the verification, before the tag move: `git archive <release-sha>` and assert `internals/detect/src/detect.py` (and, generally, the relative-path targets `action.yml` reaches) are present in the archive. That literally simulates the action-fetch mechanism against the exact commit being promoted, closing the realistic remainder without touching the workflow's derivation at all.

**Two execution invariants:**

- **"Narrow" scopes `detect`, never the suite.** Verification is still the *full* self-test + dogfood surface — every fixture, every rule job — just run version-pinned rather than hermetic. Narrowing means only that `detect` stays at the current `@v0` (per the unconstructable argument above), not that fewer jobs run.
- **Pin to the release SHA, not "current `main`."** A `workflow_run`-triggered verification checks out whatever the default branch is at trigger time; a commit landing between publish and verification would have it verify a workflow file that is *not* the one the tag will bless. Local `uses:`/`./` reusable-workflow calls resolve at the verify workflow's own commit and their refs cannot be an expression, so the mechanism that pins both the workflow file *and* its inner `uses:` to an arbitrary commit is a `workflow_dispatch` targeting the release commit. `workflow_dispatch` takes a branch or tag ref, never a bare SHA, so verification creates a **throwaway tag at the release SHA** (`verify-release-<sha>`, cleaned up in a `finally`; no workflow triggers on `push: tags:`, so creating it fires nothing), dispatches the self-test and dogfood workflows at that tag with `version: <just-published>`, and polls their conclusions — pinning each dispatched run to the exact release commit, the same forward-only discipline `move-major-tag` applies to `@v0` itself.

**Mechanism.** The direct `Release`-success → `move-major-tag` chain becomes `Release`-success → **verify-and-promote**. On a successful publish, verify-and-promote: (1) resolves the release SHA and the just-published npm version from the tags reachable there; (2) runs the layout check against the release SHA; (3) dispatches `testing-conventions-selftest.yml` and `dogfood.yml` at a throwaway tag on the release SHA with the pinned `version`, and polls until both conclude; (4) advances `@v0` via the unchanged forward-only `move_major_tag.py` **only** when the layout check and both dispatched runs are green. Every non-trivial step is the colocated-tested `tc-checks verify-release` command (`internals/checks`, `checks/utils/verify_release.py` behind an injected git/`gh` boundary — the `build-hermetic-cli` pattern, so the genuinely-equivalent boundary/timing mutants carry reasoned `testing-conventions.toml` exemptions rather than living where none is possible); the workflow YAML wires triggers, checkouts, and env, and holds no logic. It lives in `internals/checks` rather than `.github/scripts` because its `gh` boundary can't be exercised for real in CI, so a handful of mutants need exemptions the `.github/scripts` dogfood can't grant. The `rolling-release-wired`/`verify-release-wired` static checks guard that the tag move stays gated on verification, so a regression that re-introduces a bare publish-only promotion fails the self-test.

## Dogfooding the `.github/` helpers

The `.github/` helper scripts are first-party Python and are held to the same conventions as the shipped packages, not waved through: `dogfood-github-helpers.yml` runs `unit colocated-test`, `unit lint`, `unit coverage`, and `integration lint` whole-tree, plus the diff-scoped `unit mutation --base origin/main` (the published binary, via `npx`), over `.github/scripts/`. So `move_major_tag.py` carries a colocated unit test, stays isolated, meets the coverage floor, passes the mocking lint, and kills every mutant on the lines a PR touches — like any package source. It keeps its colocated unit test next to the source and its integration/e2e suites under `tests/` (uniquely named, so `unit coverage`'s pytest collects them without an import-mode flag). `detect.py` used to be dogfooded here too, until #363 migrated it into `internals/detect/` — see "The detect action's package" below.

Mutation is diff-scoped, exactly like the reusable workflow's gate on the packages: whole-tree mutation is too slow to gate, so only survivors on `<base>...HEAD` changed lines count (`base` is `origin/main`; a run with no changed helper source reports no survivors and passes). The Python mutation arm is driven by the adapter shipped in the `testing-conventions` **wheel** (`python3 -m testing_conventions.mutation.main`, #248), so the job installs the wheel — which brings cosmic-ray — alongside pytest and coverage. The wiring is guarded in CI (`github-helpers-wired` in `testing-conventions-selftest.yml`): a regression that drops any of the five arms fails the self-test. That guard runs `uv run --project internals/checks tc-checks github-helpers-wired` — the check migrated into the `internals/checks` package (#329), whose pure `wires_github_helpers` is true only when the dogfood workflow invokes all five arms — rather than inline workflow bash (extracted per epic #302, sub-issue #310; consolidated per #321).

The self-test's own output and wiring assertions no longer live here — they have all migrated into the `internals/checks` package (see "The self-test checks package" below). This section now covers only `move_major_tag.py`, the one first-party helper still dogfooded as a loose script.

The scan is scoped to the helper *code* by design. `.github/selftest/**` is **excluded** — those are intentional negative fixtures (below-floor suites, surviving mutants, un-colocated reds) the rules are *meant* to fire on — and the workflow YAML has nothing to scan. Pointing the rules at the repo root instead would need a real detection exclude/ignore (the root is full of negative fixtures under `packages/**/tests/fixtures/**` and `.github/selftest/**`, plus generated trees like `packages/python`'s build-time `.rs`, cf. #206); that mechanism is its own work, not what this gate does.

## The self-test checks package (`internals/checks`)

The #302 wiring/assertion checks are consolidated into a single uv package at `internals/checks/` — `pyproject.toml` + `uv.lock` + a `src/checks/` layout (epic #321, complete). `checks/cli.py` is a `@click.group()` (`tc-checks`) that composes each check as a subcommand; each check lives in its own subpackage — `checks/<check>/cli.py` holds a pure predicate (or, for the failure-path group, a hardcoded `CHECKS` list) and a `@click.command()`, with a colocated `cli_test.py`. Shared code lives in `checks/utils/`: `check_failed.py` (the `CheckFailed` `click.ClickException` that prints a `::error::` annotation), `run_checks.py` + `failure_reason.py` (the failure-path orchestrator and its exit-code decision), and `job_block.py` (isolating a named job's YAML region). A self-test job runs `uv run --project internals/checks tc-checks <check>`.

The layout mirrors `packages/python`, whose importable package sits in `packages/python/python` while `packages/python/tests` holds the integration/e2e suite: `source` for the dogfood points at the **inner** `internals/checks/src`, not the package root, so the static gates recurse only the source tree. The colocated `cli_test.py` units drive each check's `@click.command` through its `.callback` (no `CliRunner`, which is a third-party collaborator the isolation lint flags) and import only the unit under test — so the colocated suite alone reaches the 100% coverage floor. The full e2e suite (`CliRunner` over the real workflow file) lives at `internals/checks/tests/e2e`, a sibling **outside** the scanned `src/`; a `*_test.py` e2e file *inside* the scan would be read as an un-isolated unit test and red the lint. The package root (`internals/checks`, where the `pyproject.toml` lives) is still derived for the coverage/mutation venv.

The packaging gate's `packaging_build` derivation covers `internals/checks` too (a plain `uv build`, #335), so the dogfood packaging job builds this package's own distributions and scans them — and both must exclude the colocated `*_test.py` units the same way any other zero-config Python package would, or the scan rejects the artifact as shipping its tests (#354). `uv build` produces a wheel *and* an sdist, and hatchling's `[tool.hatch.build.targets.wheel]` / `[tool.hatch.build.targets.sdist]` exclude independently of each other — an exclude scoped to only the wheel target leaves the sdist (`.tar.gz`) shipping every test file untouched. The top-level `[tool.hatch.build] exclude = ["**/*_test.py"]` applies to both targets at once. Tests still run from the source tree (`.venv`/`uv run pytest`), never from a built artifact, so the exclude has no effect on execution — only on what `uv build` packages.

It lives under `internals/`, **not** `.github/scripts/`, on purpose: `dogfood-github-helpers.yml` scans `.github/scripts/` as *loose* first-party scripts, and a `pyproject.toml` inside that scan flips the conventions tool into package-mode for the whole directory (it stops recognizing the loose scripts' `tests/integration` / `tests/e2e` as non-unit). As a real package it is instead dogfooded through the **shipped reusable workflow** (`dogfood.yml`, `path: internals/checks/src`) — colocated-test, isolation, coverage, integration-lint, and diff-scoped mutation — exactly like `packages/python`. `dogfood-github-helpers.yml` now covers exactly one genuinely loose helper — `move_major_tag.py` (every #302 check has migrated into the package, and `detect.py` into `internals/detect`), so `.github/scripts/` holds a single directory.

## The detect action's package (`internals/detect`)

`detect.py` (the `detect` composite action's implementation, #189/#277 onward) moved out of `.github/actions/detect/` into its own uv package, `internals/detect/` (#363), mirroring `internals/checks` for the same reason: custom logic under `.github/` is loose-script territory, and adding a `pyproject.toml` there would have flipped `dogfood-github-helpers.yml`'s scan into package-mode. `internals/detect/src/detect.py` is a single top-level module (no subpackage — one file, no CLI subcommands to compose), with its colocated `detect_test.py` beside it and the integration/e2e suites at `internals/detect/tests/`, a sibling outside `src/`, exactly like `internals/checks`.

Unlike `internals/checks`, it is **not** dogfooded through the shipped reusable workflow. `internals/checks`' colocated `cli_test.py` units alone reach the coverage floor, so scoping the dogfood job's `source` to `src/` (excluding the e2e suite entirely) works cleanly. `detect.py`'s colocated `detect_test.py` alone does not — `compute_outputs`'s orchestration is exercised only by the integration suite (filesystem mocked) and the full script only by the e2e suite. Scoping to `internals/detect/src` alone therefore fails the coverage floor (the integration/e2e suites are outside the scan and never run); scoping to the package root instead (`internals/detect`, so all three tiers run together) fails `unit lint`'s `unmocked-collaborator` rule, because that rule has no concept of test tiers — once a first-party package is declared (any `pyproject.toml`), it flags *every* `*_test.py` under the scanned root that imports the package unmocked, `detect_integration_test.py` and `detect_e2e_test.py` included. (This also explains why `detect.py` silently passed `dogfood-github-helpers.yml`'s isolation check for years despite the same nested layout: `.github/actions` never had a `pyproject.toml`, so the rule's first-party-package lookup found nothing and reported no violations at all — not because the layout satisfied it.) `detect.py` keeps its existing, proven test-quality gate instead: `detect-action.yml`'s dedicated pytest run across all three tiers together (100% coverage via plain `coverage.py`), independently of this tool's own gates.

`.github/actions/detect/action.yml` is unaffected by the move — it is a thin composite-action manifest, not Python, and it is the file every consumer's `uses: …/actions/detect@v0` reference resolves against. Its `run:` step now points at `$GITHUB_ACTION_PATH/../../../internals/detect/src/detect.py`: GitHub Actions checks out the *whole* repo at the pinned ref to resolve a composite action (not just the action's own subdirectory), so a relative path climbing back out to the repo root and down into `internals/` resolves identically whether the action is used locally (`./.github/actions/detect`) or externally (`owner/repo/.github/actions/detect@ref`). The `uses:` contract itself never changes, so this is not a breaking change for any consumer and needs no `MIGRATIONS.md` entry.

With epic #321 complete, every #302 wiring/assertion and failure-path check lives in `internals/checks` as a `tc-checks <check>` subcommand; the flat `.github/scripts/<check>/` dirs are gone, and each self-test job invokes `uv run --project internals/checks tc-checks <check>` after `astral-sh/setup-uv`. The full inventory, by original sub-issue:

- **Wiring assertions (#323):** `mutation-wired`, `isolation-wired`, `coverage-rust-wired`, `colocated-rust-wired`, `diff-scoped-wired`, `e2e-verify-wired`, `e2e-verify-checks-out-pr-head` (block-scoped to the `e2e-verify` job, replacing the old `awk` range), `e2e-verify-scope-wired`, `rolling-release-wired` (two selftest steps folded into one command over two file arguments).
- **Detect wiring (#324):** `wiring-detect-action`, `wiring-packaging-default-on`, `wiring-e2e-default-on`, and `detect-routes-python` — the last keeps its `uses: ./.github/actions/detect` step in the job and passes the action's `isolation_languages` output as a single-quoted JSON CLI argument.
- **Feature-input wiring (#325):** `build-command-wired`, `gates-wired`, `rust-toolchain-wired`.
- **Package-root wiring (#326):** `coverage-package-root-wired`, `packaging-package-root-wired`, `mutation-package-root-wired` — each isolates a job's YAML region and asserts it references `needs.detect.outputs.package_root`.
- **Detect-output validations (#327):** `detect-package-root-ts`, `detect-package-root-py` — each runs `./.github/actions/detect` against a monorepo fixture and hands the outputs to a pure `evaluate` returning the first mismatch's message.
- **Failure-path (#328):** `isolation-red`, `below-floor`, `mutation-gate`, `python-mutation-clean`, `packaging-red`, `coverage-rust-red`, `integration-lint-new-arms-trip`, `packaging-package-root-red`, `colocated-rust-red` (#379) — each runs hermetic-CLI (`./hermetic-cli/testing-conventions`, from `config.HERMETIC_CLI`) invocations from a `CHECKS` list and asserts the exit code via `failure_reason`.
- **github-helpers-wired (#329).**
- **red-path-hermetic-wired (#379):** asserts every failure-path job downloads the `hermetic-cli` artifact (`needs: [build-cli]` + `./.github/actions/download-hermetic-cli`), so none drives npm-latest.

The static checks hold their inspection in a pure predicate over the workflow file; the failure-path group holds a `CHECKS` list run through the shared `run_checks` orchestrator. Either way the colocated `cli_test.py` drives the pure logic in isolation, the `@click.command()` raises `CheckFailed` (a `::error::` annotation) on a failure, and a sibling `CliRunner` e2e suite exercises the real boundary — held to the same coverage and mutation bar as any shipped source.

The two pre-existing first-party helpers were resolved per the #321 open question: `detect.py` moved to `internals/detect` (#363), while `move_major_tag.py` stays a loose script under `.github/scripts/` — it wires the tag-move workflow, has no CLI subcommands to compose into the checks group, and is already held to the full bar by `dogfood-github-helpers.yml`. `.github/scripts/` therefore holds exactly one directory, and the epic is closed.

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
