# Reference

Information-oriented reference for the `testing-conventions` CLI and its config file.

## CLI

```
testing-conventions <COMMAND>
```

Global flags: `--help`, `--version`.

### `unit colocated-test`

Check that every source file under a directory has a colocated unit test. `colocated-test` is
one rule under the `unit` command group; sibling rules (`unit coverage`, `unit isolation`) and
other test kinds (`integration`, `e2e`) nest the same way.

```
testing-conventions unit colocated-test --language <LANG> [--config <CONFIG>] <PATH>
```

| Argument / flag     | Description                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `<PATH>`            | Directory to scan recursively.                                    |
| `--language <LANG>` | **Required.** Convention to enforce: `python`, `typescript`, or `rust`. No default — omitting it is a usage error, never a silent `python` run. |
| `--config <CONFIG>` | Config file supplying the `exempt` list (default `testing-conventions.toml`). Optional — if the file is absent, nothing is exempt. |

**What counts, by language:**

- **`python`** — a source `*.py` needs a colocated `*_test.py`. `*_test.py` files are the tests
  themselves. Note `__init__.py` is **not** auto-exempt: an empty one is skipped (see below),
  but a non-empty one needs a test or an [exemption](#exemptions).
- **`typescript`** — a source `*.ts` / `*.tsx` / `*.mts` / `*.cts` needs a colocated
  `*.test.*` of the matching extension (`foo.mts` → `foo.test.mts`). `*.test.*` files are the
  tests; declaration files (`*.d.ts` / `*.d.mts` / `*.d.cts`) carry no runtime code and are
  ignored.
- **`rust`** — units are inline `#[cfg(test)]` modules, not sibling files, so the check is
  *presence*: a `src` file that defines a function with a body but has no inline `#[cfg(test)]`
  module is an orphan. Module-declaration files (a `lib.rs`/`mod.rs` of only `mod`/`use`) and
  type-only files (no `fn`) aren't subjects; integration crates under `tests/` (and `benches/` /
  `examples/` / `build.rs`) are skipped.

Two things are not subjects regardless of language: **empty or comment-only files** (no logic
to test) and any file listed in the config [`exempt`](#exemptions) table.

**Exit codes:**

| Exit | Meaning                                                                                          |
| ---- | ----------------------------------------------------------------------------------------------- |
| `0`  | Every source file has its colocated unit test. Nothing is printed.                              |
| `1`  | One or more orphans, each printed to stderr (`missing colocated unit test: <path>`; for `rust`, `missing inline #[cfg(test)] tests: <path>`), then a count. |

### `unit coverage`

Run the unit suite under coverage and fail if it's below the floor.

```
testing-conventions unit coverage --language <LANG> [--config <CONFIG>] <PATH>
```

| Argument / flag     | Description                                                                |
| ------------------- | -------------------------------------------------------------------------- |
| `<PATH>`            | Directory whose unit suite is run and measured.                            |
| `--language <LANG>` | **Required.** `python` or `typescript` (Rust coverage is a separate item). |
| `--config <CONFIG>` | Config file providing the thresholds and `exempt` list (default `testing-conventions.toml`). Optional — if the file, or its `[<language>].coverage` table, is absent, the language's default floor is used and nothing is exempt. |

With no `[<language>].coverage` table — or no config file at all — the check uses the language's
**default floor**, the reasonable one from the internals style guides: Python
`branch = true, fail_under = 85`; TypeScript `lines = 80, branches = 75, functions = 80,
statements = 80`. A config table overrides it. This is what lets the [reusable workflow](../guide/ci)
opt a new library into coverage with no config file.

For **`python`**, runs `coverage.py` with branch coverage on — measuring the sources under
`<PATH>` with `*_test.py` excluded from the denominator — and compares the total against
`[python].coverage` (`fail_under`, `branch`). Exits `0` when the floor is met, `1` (with the
actual vs. required percent on stderr) when it isn't. `coverage` and `pytest` must be installed.
Files with a `coverage` [exemption](#exemptions) are also excluded from the denominator.

For **`typescript`**, runs `vitest` with v8 coverage (the json-summary reporter) — measuring the
`.ts` / `.tsx` / `.mts` / `.cts` sources under `<PATH>` with `*.test.*` and declaration files
excluded from the denominator — and compares each of the four metrics against
`[typescript].coverage` (`lines`, `branches`, `functions`, `statements`). Exits `0` when every
floor is met, `1` (naming each metric below its floor on stderr) when any isn't. The tool invokes
`npx vitest`, so `vitest` and `@vitest/coverage-v8` must be installed under `<PATH>`. Files with a
`coverage` [exemption](#exemptions) are also excluded from the denominator.

### `unit isolation`

Check that unit tests isolate the unit under test — collaborators are mocked (Python, TypeScript)
or never called out to (Rust). A unit test that touches a real collaborator is an integration test
wearing a unit's name.

```
testing-conventions unit isolation --language <LANG> [--config <CONFIG>] <PATH>
```

| Argument / flag     | Description                                                                            |
| ------------------- | ------------------------------------------------------------------------------------- |
| `<PATH>`            | Directory to scan recursively (for Rust, the crate root, whose `Cargo.toml` names the external crates). |
| `--language <LANG>` | **Required.** `python`, `rust`, or `typescript`.                                       |
| `--config <CONFIG>` | Config file supplying the `exempt` list (waivers). Optional (default `testing-conventions.toml`); if absent, nothing is waived. |

Reports each violation to stderr as `path:line: <rule> — <message>` and exits `1` if any are
found, `0` otherwise. Any rule below is **waivable** per file via a reason-required
[`exempt`](#exemptions) entry (`rules = ["no-out-of-module-call"]`, etc.).

**Rust** — parses each `*.rs` file under the crate root with `syn` and walks its inline
`#[cfg(test)]` modules:

- **`no-out-of-module-call`** — a call out of a unit test's own module: `crate::…` (another
  first-party module), `super::super::…` (an ancestor), an external crate (named in `Cargo.toml`;
  `[dev-dependencies]` like `mockall` are test tooling and excluded), or effectful `std`
  (`fs` / `net` / `process` / `env` / `thread` / `os`, the clock, or real-handle I/O). A single
  `super::` (the unit under test), `self` / `Self`, a bare unqualified call, and pure `std` —
  including `std::io::Cursor` and the I/O traits — stay in-module. Inject a trait double
  (hand-rolled or `mockall`) for a collaborator instead.
- **`no-out-of-module-import`** — a `use` inside a test module that brings in a foreign surface:
  a glob of anything but `super::*`, or a named import rooted at `crate::`, an external crate, or
  effectful `std`. `use super::*` / `use super::Thing` (the unit under test), `self`, and pure
  `std` (`std::collections::HashMap`, `std::io::Cursor`, …) are in-module. This catches a
  collaborator that's imported and then called unqualified, which the call check can't see.

Full name-resolution precision — a collaborator reached through an unqualified call, a
`use … as …` rename, or a macro — is a future `dylint` pass; the `syn` heuristic is the
deterministic bright-line.

**TypeScript** — parses each `*.test.{ts,tsx,mts,cts}` file with the `oxc` parser:

- **`unmocked-collaborator`** — any runtime import that isn't `vi.mock()` / `vi.doMock()`-ed.
  Three imports are never collaborators and so are never flagged: the **unit under test** (the
  colocated source, `widget.test.ts` → `./widget`, imported and run for real), **type-only**
  imports (`import type …` — erased at compile time), and the **test runner** (`vitest` /
  `@vitest/*`). This is the unit-suite mirror of [`integration lint`](#integration-lint)'s
  `no-first-party-mock`; see the [Isolation guide](../guide/isolation).
- **`untyped-mock`** — a `vi.mock(spec, factory)` whose factory has no `vi.importActual<…>()`
  type anchor, so the double can drift from the real module. Anchor it with
  `vi.importActual<typeof import(spec)>()` (the README pattern). A bare `vi.mock(spec)`
  (vitest auto-mock, typed from the real module) and an already-typed factory both pass.

**Python** — parses each colocated unit test (`*_test.py` / `test_*.py`, not `conftest.py`) with
the Rust Python parser:

- **`unmocked-collaborator`** — an imported **first-party** collaborator that the test doesn't
  mock. First-party is the dist's own package (read from the nearest `pyproject.toml`
  `[project].name`, as in [`integration lint`](#integration-lint)'s `no-first-party-patch`), or a
  relative import. Three things are never collaborators: the **unit under test** (the import whose
  module's last segment matches the test's base name — `widget_test.py` ↔ `myproject.widget`), the
  **test framework** (`pytest` / `unittest` / `unittest.mock`), and **pure stdlib** / `__future__` /
  `TYPE_CHECKING`-guarded (type-only) imports. An import counts as **mocked** when a `patch("…")` in
  the file targets a matching last segment — `patch("myproject.widget.record")` mocks an imported
  `record` (the convention patches the name in the consuming module). The canonical unit test
  imports only the unit under test and patches collaborators by string, so it has no collaborator
  imports to flag. Un-mocked third-party / effectful-stdlib imports are a separate slice; a
  first-party *value/type* import used to build test data is a documented non-goal (waive it). See
  the [Isolation guide](../guide/isolation).

## Exemptions

Not every source file should need a colocated test or full coverage — a launcher shim, a pure
re-export barrel, generated code. So the checker can be a *blocking* gate without forcing
pointless tests, files are exempted **explicitly, in the config**. There is no automatic name-
or shape-based exemption — the only files skipped automatically are those with no logic at all.

### Empty files (automatic)

A file with no code — empty, or only whitespace and comments — has nothing to test and is never
a subject. This is the only automatic exclusion, and it's why a bare `__init__.py` needs no
configuration. (A declaration file `*.d.ts` is likewise never tracked: it carries no runtime
code.) The moment a file gains a statement — a re-export, a constant, a function — it becomes a
subject and needs a colocated test or an entry below.

### The `exempt` list (explicit, reason-required)

For a deliberate omission, add a `[[<language>.exempt]]` entry to the config:

```toml
[[python.exempt]]
path = "mypkg/cli.py"          # relative to the scanned <PATH>
rules = ["colocated-test", "coverage"]  # which checks this lifts
reason = "thin launcher; logic in run(), tested in run_test.py"  # required
```

| Field | Meaning |
| ----- | ------- |
| `path` | The exempt file, relative to the scanned `<PATH>`. Must point to a file that exists — a stale entry is a hard error, so the list can't silently rot. |
| `rules` | Which checks the exemption lifts: `colocated-test`, `coverage`, a mocking lint (`no-monkeypatch`, `no-inline-patch`, `no-environ-mutation`, `no-constant-patch`, `no-first-party-patch`), or an isolation rule (`no-out-of-module-call`, `no-out-of-module-import`, `no-first-party-double`, `unmocked-collaborator`, `untyped-mock`, `no-first-party-mock`). |
| `reason` | Why the omission is deliberate. **Required** — an empty reason is rejected on load. |

Because every exemption lives in the one config file, names its rules, and carries a reason,
the project's entire exemption surface is auditable in a single diff — the opposite of a prose
omit-list or a scattered set of ignore comments. A re-export barrel (`index.ts`), a launcher
shim, or a non-empty `__init__.py` is exempted this way, not automatically.

### `integration lint`

Lint integration test files for mocking mechanism & style. The first rule group under the
`integration` command; future lints join it under the same command.

```
testing-conventions integration lint --language <LANG> [--config <CONFIG>] <PATH>
```

| Argument / flag     | Description                                                        |
| ------------------- | ----------------------------------------------------------------- |
| `<PATH>`            | Directory to scan recursively for test files.                     |
| `--language <LANG>` | **Required.** `python`, `typescript`, or `rust`. Omitting it is a usage error. |
| `--config <CONFIG>` | Config file supplying the `exempt` list (waivers). Optional (default `testing-conventions.toml`); if absent, nothing is waived. |

Reports each violation to stderr as `path:line: <lint> — <message>` and exits `1` if any are
found, `0` otherwise.

**Python** — parses each test file (`*_test.py`, `test_*.py`, `conftest.py`) with a Rust Python
parser and walks the AST:

- **`no-monkeypatch`** — a test or fixture function that declares the `monkeypatch` parameter.
  pytest's `monkeypatch` is banned; patch with `unittest.mock` (`patch` / `patch.object` /
  `patch.dict`) wrapped in a `pytest.fixture` instead. **Waivable** per file via
  `rules = ["no-monkeypatch"]` (reason required).
- **`no-inline-patch`** — a `patch(...)` / `patch.object(...)` / `patch.dict(...)` call in a
  test body, whether the `with patch(...)` form or a bare call. Move the patch into a
  `pytest.fixture`; a patch inside a fixture is allowed. **Waivable** via
  `rules = ["no-inline-patch"]`.
- **`no-environ-mutation`** — direct mutation of `os.environ`: `os.environ[...] = …`,
  `del os.environ[...]`, or a mutating method (`update` / `pop` / `setdefault` / `clear` /
  `popitem`). Set env via `patch.dict(os.environ, {...})`; reading `os.environ` is fine.
  **Waivable** via `rules = ["no-environ-mutation"]`.
- **`no-constant-patch`** — patching a module-global UPPER_CASE constant, e.g.
  `patch("pkg.config.CACHE_DIR", …)`. Inject the config explicitly instead. **Waivable**
  per file: add a `[[python.exempt]]` entry with `rules = ["no-constant-patch"]` (and a
  reason) and pass it via `--config`; a waived file is silent.
- **`no-first-party-patch`** — a `patch(...)` whose string target is **first-party**, e.g.
  `patch("ourpkg.mod.fn")`. An integration test runs first-party code for real, so only
  third-party packages (`requests.get`) and effectful stdlib (`subprocess.run`,
  `builtins.open`) may be patched. The dist's own top-level package is read from the nearest
  `pyproject.toml` `[project].name` (normalized to an import name); a tree with no declared
  package flags nothing. `patch.object(module, …)` and non-literal targets are left alone.
  **Waivable** via `rules = ["no-first-party-patch"]`. See the
  [Isolation guide](../guide/isolation).

**TypeScript** — parses each test file (`*.test.{ts,tsx,mts,cts}`) with the `oxc` parser and
walks the AST:

- **`no-first-party-mock`** — a `vi.mock()` / `vi.doMock()` whose target is a **first-party**
  module (a relative specifier like `./service` or `../core`). An integration test runs
  first-party code for real, so only third-party packages (`stripe`) and Node built-ins
  (`node:fs`, `child_process`) may be mocked. A non-literal target (`vi.mock(name)`) can't be
  classified deterministically and is left alone. See the [Isolation guide](../guide/isolation).

**Rust** — parses each `*.rs` file in a `tests/` directory (the integration crates) with `syn`:

- **`no-first-party-double`** — a `#[double]` (mockall_double) import of a **first-party** item:
  the crate under test (its `Cargo.toml` `[package].name`) or a `path` dependency. An
  integration test runs first-party code for real, so only external crates and `std` may be
  doubled. `crate::` here is the integration-test crate itself (not the library under test), so
  it isn't flagged. This is the integration mirror of [`unit isolation`](#unit-isolation)'s
  out-of-module rules; full precision (renames, `mock!` macros) is a future `dylint` pass.

### `e2e attest`

Run the e2e suite locally and record that it ran against the current commit — the *write* half
of the e2e attestation nudge. The first command under the `e2e` group; the CI-side `e2e verify`
gate follows.

```
testing-conventions e2e attest '<command>'
```

| Argument | Description |
| -------- | ----------- |
| `<command>` | The e2e command to run (e.g. `pnpm run e2e`), executed via the shell with its output streamed through. |

Run from the repository root. `attest` resolves the current commit (`HEAD`), runs `<command>`
capturing its exit code, writes `e2e-attestation.json` recording the command, a timestamp, the
exit code, and the commit SHA it was run against, and commits that file on top — the attestation
names the code commit beneath it, since a commit can't name its own SHA.

It writes **regardless of the command's exit code** — the point is to force a *run*, not a
*pass* — and exits `0` once the attestation is recorded and committed. The companion `e2e verify`
(a CI gate confirming the latest code commit is attested) is not shipped yet.

### `e2e verify`

The CI side of the e2e attestation nudge: confirm the committed attestation names the current
code, **without ever running e2e**. Pairs with [`e2e attest`](#e2e-attest).

```
testing-conventions e2e verify
```

Run from the repository root. `verify` reads `e2e-attestation.json` and passes (exit `0`) iff its
recorded SHA equals the **latest code commit** — the newest commit that changed any path other
than the attestation file itself. Otherwise it exits non-zero with an actionable message naming
the fix, e.g. *"e2e attestation out of date … run `testing-conventions e2e attest '<your e2e
command>'`"*. It never inspects the recorded exit code or output — presence + freshness only.

Push new code without re-attesting and the recorded SHA no longer names the latest code commit, so
`verify` fails until you re-run `attest`. That staleness is the nudge.

### `check`

Reserved for the config-driven umbrella that runs every configured rule. **Not wired yet** —
it currently exits `0`. Rules ship under their test-kind group (like `unit colocated-test`)
until `check` orchestrates them from the config.

### `packaging`

Confirm a built artifact doesn't ship test files (README "Packaging"). Colocated unit tests
live next to the source, so packaging has to strip them; this rule inspects the built artifact
and fails if any test file survived.

```
testing-conventions packaging --language <LANG> <PATH>
```

| Argument / flag     | Description                                                                       |
| ------------------- | --------------------------------------------------------------------------------- |
| `<PATH>`            | The built artifact to inspect: a Python wheel (`.whl`) or sdist (`.tar.gz`), a TypeScript `npm pack` tarball (`.tgz`), a Rust `cargo package` crate (`.crate`), or a directory (an already-unpacked artifact, e.g. a `dist/` tree). |
| `--language <LANG>` | **Required.** `python`, `typescript`, or `rust`.                                  |

Scans the artifact recursively for the language's test-file pattern and exits `0` when none
are present, `1` (printing each offending path, relative to the artifact root) when one is:

- **`python`** → `*_test.py` (in the wheel or sdist).
- **`typescript`** → `*.test.*` (in the `npm pack` tarball's `dist/`).
- **`rust`** → the crate-root **`tests/`** directory (in the `.crate`). Inline `#[cfg(test)]`
  units compile out of the consumer artifact for free; only the integration `tests/` needs a
  Cargo `exclude`. (Patterns ending in `/` match a directory; the others are file-name globs.)

A `.whl` (zip), or a `.tgz` / `.tar.gz` / `.crate` (gzipped tar), is unpacked first, then
scanned; a directory is scanned in place.

**Status:** all three languages land: Python wheel + sdist (#72, #106), TypeScript `npm pack`
tarball (#73), Rust `.crate` (#74). `<PATH>` may also be an already-unpacked directory.

### workflow

Guard a CI workflow against CLI subcommand drift. The reusable workflow's documented
consumption path is `…/testing-conventions.yml@v0`: the workflow file is frozen at the tag
while a consumer's `npx` pulls the latest published binary, so a renamed or removed
subcommand strands every `@v0` consumer (as it did at 0.0.7, broken by the
`location` → `colocated-test` rename). This check fails the build the moment a workflow
invokes a subcommand the binary no longer exposes.

```
testing-conventions workflow <PATH>
```

| Argument / flag | Description                                                                  |
| --------------- | ---------------------------------------------------------------------------- |
| `<PATH>`        | A workflow file, or a directory scanned recursively for `*.yml` / `*.yaml`.  |

Finds every `testing-conventions …` invocation in the workflow's shell — the bare
`npx -y testing-conventions` / on-`PATH` command word, version pin and all — and checks each
one's subcommand chain against the binary's own command tree (the source of truth). Reports
each offending invocation to stderr as `path:line: no-unknown-subcommand — <message>` and
exits `1` if any are found, `0` otherwise.

## Configuration

The standard is config-driven: one TOML file is the single source of truth for every rule's
thresholds and exemptions. The schema is validated by the loader (unknown keys, malformed TOML,
and reason-less `exempt` entries are rejected). Each `[python]` / `[typescript]` / `[rust]`
table is optional, and within it both `coverage` and `exempt` are optional — a repo can
configure just coverage, just exemptions, or both.

```toml
[python]
coverage = { branch = true, fail_under = 100 }

# A deliberate, reason-required omission (see Exemptions above):
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test", "coverage"]
reason = "thin launcher; logic in run(), tested in run_test.py"

[typescript]
coverage = { lines = 100, branches = 100, functions = 100, statements = 100 }

[rust]
coverage = { regions = 100, lines = 100 }
```

`[python].coverage` is consumed by `unit coverage` and the `exempt` lists by both rules; the
other coverage tables are accepted but not yet enforced (their rules are forthcoming). Each
package's `MIGRATIONS.md` carries the public-API upgrade history.
