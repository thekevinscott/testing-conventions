---
description: The testing-conventions.toml schema — every coverage key and its default, the exemption schema, and the shared test configs.
---

# Configuration

One TOML file is the single source of truth for what the rules require: coverage floors and
reason-required exemptions. This page is the canonical record of its schema and every default.
For the task, see [Configure the rules](../guide/configure); for the design, [Scoping and
exemptions](../explanation/scoping).

The file is named by the workflow's `config` input (default `testing-conventions.toml`, resolved
per call — a [monorepo](../monorepo) passes one per package). The loader validates the schema:
unknown keys, malformed TOML, and reason-less `exempt` entries are rejected. Each `[python]` /
`[typescript]` / `[rust]` table is optional, and within it both `coverage` and `exempt` are
optional.

```toml
[python]
coverage = { branch = true, fail_under = 100 }

# A whole-file presence exemption:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test"]
reason = "thin launcher; logic in run(), tested in run_test.py"

# A line-scoped coverage/mutation exemption — `lines` is required, and never
# shares an entry with a whole-file rule:
[[python.exempt]]
path = "mypkg/config/tomlcompat.py"
rules = ["coverage", "mutation"]
lines = [9, 10, "12-13"]
reason = "version-conditional tomllib/tomli import; one branch is dead on any single interpreter"

[typescript]
coverage = { lines = 100, branches = 100, functions = 100, statements = 100 }

[rust]
# Cargo features the suite-running Rust rules enable, so `#[cfg(feature = ...)]`
# code is compiled, measured, and mutated:
features = ["cli"]
coverage = { regions = 100, lines = 100 }
```

## Coverage

Coverage floors apply to the **unit suite only** (test files are excluded from the denominator).
Each language's default is a strict 100% floor; a `[<language>].coverage` table lowers it as a
**partial override** — set only the fields you want to move, the rest keep their default, and a
typo'd key is rejected. [Why a 100% floor](../explanation/coverage) carries the rationale,
including why Rust's extra metrics are opt-in.

| Language | Keys | Default |
| --- | --- | --- |
| **Python** | `branch`, `fail_under` | `branch = true`, `fail_under = 100` — coverage.py's combined line + branch total. |
| **TypeScript** | `lines`, `branches`, `functions`, `statements` | All four at `100`, each enforced independently. |
| **Rust** | `lines`, `regions`, `functions`, `branch` | `lines = 100`; `regions`, `functions`, and `branch` are opt-in floors. A `branch` floor adds `--branch` to the `cargo llvm-cov` run, which needs a nightly toolchain (pin one in `rust-toolchain.toml` with `llvm-tools-preview`, or set a rustup directory override); on stable the run fails with the requirement named. |

`unit mutation` has **no percentage key** — the gate is binary, not a score, and config can't
loosen it. Its only tuning is a line-scoped `mutation` exemption (below); see
[Why mutation testing](../explanation/mutation#why-a-number-wont-do-equivalent-mutants).

## Exemptions

A deliberate omission is a `[[<language>.exempt]]` entry:

| Field | Meaning |
| ----- | ------- |
| `path` | The exempt file, **relative to the scanned `path`** of the call that loads this config — except for `integration lint`'s suite subjects, which resolve **relative to the [package root](../monorepo#everything-derives-from-the-package)** the tiers derive from (e.g. `tests/integration/billing_test.py`). Must point to a file that exists; a stale entry is a hard error, so the list can't silently rot. |
| `rules` | Which checks the exemption lifts: `colocated-test`, `coverage`, `co-change`, `mutation`, a mocking lint (`no-monkeypatch`, `no-inline-patch`, `no-environ-mutation`, `no-constant-patch`, `no-first-party-patch`), an isolation rule (`no-out-of-module-call`, `no-out-of-module-import`, `no-first-party-double`, `unmocked-collaborator`, `untyped-mock`, `no-first-party-mock`), or the suite-layout rule (`unknown-tier`). |
| `lines` | The lines a `coverage` / `mutation` exemption covers. **Required** with `coverage` / `mutation`, **rejected** with any other rule. |
| `reason` | Why the omission is deliberate. **Required**: an empty reason is rejected on load. |

### Line-scoped exemptions

The measured-line rules — `coverage` and `mutation` — are never whole-file: their entries carry a
`lines` list naming the exact lines they cover. Each element is a 1-based line number (a TOML
integer) or an inclusive range (a `"start-end"` string). A **determinism guard** checks the list:

- A listed line that **isn't failing** — covered (`coverage`), its mutants all caught
  (`mutation`), or carrying no measured code — is a **hard error**.
- A failing line that **isn't listed** fails the gate as normal.

So the list is exactly the failing lines. The two entry kinds never mix: an entry naming
`coverage` / `mutation` without `lines` is rejected on load, a `lines` key alongside a whole-file
rule is rejected, and a file exempt from both `colocated-test` and `coverage` is two entries.
Under the diff-scoped mutation job, a listed line outside the diff isn't mutated and is left
alone; the guard fires only on a listed line whose mutants were run and all caught.

### Automatic exemptions

Two kinds of files are skipped with no configuration — the only non-explicit exclusions:

- **Empty or comment-only files** — nothing to test (a bare `__init__.py`, say). The moment a file
  gains a statement, it becomes a subject.
- **Declaration files** (`*.d.ts` / `*.d.mts` / `*.d.cts`) — they carry no runtime code.

## `[rust] features`

The `[rust]` table takes **`features`**, a list of cargo features the suite-running Rust rules
enable: `unit coverage` passes it to `cargo llvm-cov` as `--features`, and `unit mutation` forwards
it to cargo-mutants' build/test runs, so `#[cfg(feature = ...)]` code is compiled, measured, and
mutated. Cargo features are Rust's build-system concept with no Python/TypeScript analog, so the
key is deliberately Rust-only — a documented asymmetry under the
[parity rule](../explanation/#parity-over-cleverness).

## `build_command`

Each language table — `[python]`, `[typescript]`, `[rust]` — takes **`build_command`**, a shell
command a build-dependent job runs after toolchain and dependency setup and **before** it builds or
imports the package. It **supplies a necessary fact** — how to build a package the ecosystem
doesn't build for you — for a build the manifest **structurally can't express**: where an ecosystem
standardizes the build, the tool derives it (a maturin/PEP 517 backend, Cargo's `build.rs` and
`cargo package`, npm's `prepare` / `prepack` run by `npm pack`), and `build_command` names only the
remainder — never a heuristic that guesses a script name that isn't standardized.

It is **not an escape hatch** and requires no justification: unlike `gates` (which skips a check) or
`rust_toolchain` (which overrides a working default), it waives nothing — it just names the build.
So it carries **no required `reason`**; an optional `reason` note is retained if you want to explain
the build, but a bare `build_command` loads.

```toml
[typescript]
build_command = "pnpm build"
```

The workflow discovers `build_command` in the package's own `testing-conventions.toml` at the
[package root](../monorepo), exactly like the config file itself — a config key, not a `uses:`-call
input, and one per language table so a package names only the build its own language needs.

Where each language reaches for it:

- **Python** — the common case: a unit suite that imports a compiled module (a maturin/PyO3
  extension) needs it built first, and PEP 517 backends expose only sandboxed `build_wheel` /
  `build_sdist` hooks with no manifest-declared pre-build shell step
  (`build_command = "uv run maturin develop"`).
- **TypeScript** — for packaging, when the compile-before-pack lives in a script npm doesn't run on
  `pack`. npm standardizes `pack` (which runs `prepare` / `prepack`) but not the build script's name
  — it's `build` in one package and `compile` in the next — so the tool can't derive it, and this
  one line names it.
- **Rust** — rarely: `cargo` compiles via `build.rs` and packages via `cargo package` from the
  manifest, so `build_command` is only for a pre-build step neither expresses.

## `[e2e] extra_scope` and `exclude`

The `[e2e]` table takes **`extra_scope`**, a list of repo-root-relative directories outside the
package's own subtree whose changes join the [`e2e verify`](../explanation/e2e#a-shared-source-tree-beside-the-package)
scoped diff, and **`exclude`**, a list of feature-gated subtrees carved back out of that union.
It is the declaration for a package whose e2e artifact is compiled from a **shared source tree that
sits beside it** — a native core bound into several language bindings — which no `--scope`
at-or-below the package root can reach.

```toml
[e2e]
extra_scope = ["packages/rust/src"]
exclude = ["packages/rust/src/cli", "packages/rust/src/bin"]
```

The value is discovered in the package's own `testing-conventions.toml` at the
[package root](../monorepo), exactly like the config file itself — a fact about the package's build
(*my artifact is compiled from that tree*), not a `uses:`-call input. `detect` renders the lists as
repeated `--extra-scope` / `--exclude` arguments and the `e2e-verify` job appends them; a package
declaring neither scopes the diff to its own `path` alone. Both are lists of directory paths, so a path
with a space is not supported. This is git-level and language-agnostic — it holds across Python,
TypeScript, and Rust identically. Like `build_command`, the tool's own config loader never acts on
these keys — it accepts the table so the rest of the config still loads — while `detect` and the
workflow read them.

## Shared test configs

The coverage floor is also published as a config your own test runner extends, so a local run is
held to the same floor CI enforces:

- **TypeScript** — the npm package exports `vitestConfig` from its root; extend it with
  `mergeConfig` from `vitest/config`. It carries the v8 provider, the `src/**` coverage scope
  (declaration files excluded), and the `100/100/100/100` thresholds. `vitest` is an optional peer
  dependency, and the import resolves to the library entry, separate from the CLI.
- **Python** — the `testing-conventions` wheel auto-loads a pytest plugin that holds a
  `pytest --cov` run to the same floor: branch coverage on, `fail_under = 100`, and test files
  (`*_test.py` / `conftest.py`) omitted from the denominator. It acts only when a coverage run is
  active, and your own setting always wins — set `branch`, `fail_under`, or `omit` in any coverage
  config location and the plugin leaves it alone.
