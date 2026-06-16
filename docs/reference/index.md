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
| `--language <LANG>` | **Required.** Convention to enforce: `python` or `typescript`. No default — omitting it is a usage error, never a silent `python` run. |
| `--config <CONFIG>` | Config file supplying the `exempt` list (default `testing-conventions.toml`). Optional — if the file is absent, nothing is exempt. |

**What counts, by language:**

- **`python`** — a source `*.py` needs a colocated `*_test.py`. `*_test.py` files are the tests
  themselves. Note `__init__.py` is **not** auto-exempt: an empty one is skipped (see below),
  but a non-empty one needs a test or an [exemption](#exemptions).
- **`typescript`** — a source `*.ts` / `*.tsx` / `*.mts` / `*.cts` needs a colocated
  `*.test.*` of the matching extension (`foo.mts` → `foo.test.mts`). `*.test.*` files are the
  tests; declaration files (`*.d.ts` / `*.d.mts` / `*.d.cts`) carry no runtime code and are
  ignored.

Two things are not subjects regardless of language: **empty or comment-only files** (no logic
to test) and any file listed in the config [`exempt`](#exemptions) table.

**Exit codes:**

| Exit | Meaning                                                                                          |
| ---- | ----------------------------------------------------------------------------------------------- |
| `0`  | Every source file has its colocated unit test. Nothing is printed.                              |
| `1`  | One or more orphans. Each prints to stderr as `missing colocated unit test: <path>`, then a count. |

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

Check that inline unit tests call nothing out of their own module (Rust). A unit test belongs to
the module it sits in; reaching a real collaborator makes it an integration test wearing a unit's
name.

```
testing-conventions unit isolation --language <LANG> <PATH>
```

| Argument / flag     | Description                                                                            |
| ------------------- | ------------------------------------------------------------------------------------- |
| `<PATH>`            | Crate root to scan recursively (its `Cargo.toml` names the external crates).          |
| `--language <LANG>` | **Required.** `rust` only for now (Python / TypeScript isolation are separate items). |

Parses each `*.rs` file under `<PATH>` with `syn` and walks its inline `#[cfg(test)]` modules,
reporting each violation to stderr as `path:line: <rule> — <message>` and exiting `1` if any are
found, `0` otherwise.

**Detector:**

- **`no-out-of-module-call`** — a call out of a unit test's own module: `crate::…` (another
  first-party module), `super::super::…` (an ancestor), an external crate (named in `Cargo.toml`;
  `[dev-dependencies]` like `mockall` are test tooling and excluded), or effectful `std`
  (`fs` / `net` / `process` / `env` / `thread` / `os`, the clock, or real-handle I/O). A single
  `super::` (the unit under test), `self` / `Self`, a bare unqualified call, and pure `std` —
  including `std::io::Cursor` and the I/O traits — stay in-module. Inject a trait double
  (hand-rolled or `mockall`) for a collaborator instead.

Full name-resolution precision — a collaborator reached through an unqualified call, a
`use … as …` rename, or a macro — is a future `dylint` pass; the `syn` heuristic is the
deterministic bright-line.

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
| `rules` | Which checks the exemption lifts: `colocated-test` (skip the colocated-test requirement) and/or `coverage` (omit from the coverage denominator). |
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
| `--language <LANG>` | **Required.** `python` or `typescript`. Omitting it is a usage error. |
| `--config <CONFIG>` | Config file supplying the `exempt` list (waivers). Optional (default `testing-conventions.toml`); if absent, nothing is waived. |

Reports each violation to stderr as `path:line: <lint> — <message>` and exits `1` if any are
found, `0` otherwise.

**Python** — parses each test file (`*_test.py`, `test_*.py`, `conftest.py`) with a Rust Python
parser and walks the AST:

- **`no-monkeypatch`** — a test or fixture function that declares the `monkeypatch` parameter.
  pytest's `monkeypatch` is banned; patch with `unittest.mock` (`patch` / `patch.object` /
  `patch.dict`) wrapped in a `pytest.fixture` instead.
- **`no-inline-patch`** — a `patch(...)` / `patch.object(...)` / `patch.dict(...)` call in a
  test body, whether the `with patch(...)` form or a bare call. Move the patch into a
  `pytest.fixture`; a patch inside a fixture is allowed.
- **`no-environ-mutation`** — direct mutation of `os.environ`: `os.environ[...] = …`,
  `del os.environ[...]`, or a mutating method (`update` / `pop` / `setdefault` / `clear` /
  `popitem`). Set env via `patch.dict(os.environ, {...})`; reading `os.environ` is fine.
- **`no-constant-patch`** — patching a module-global UPPER_CASE constant, e.g.
  `patch("pkg.config.CACHE_DIR", …)`. Inject the config explicitly instead. **Waivable**
  per file: add a `[[python.exempt]]` entry with `rules = ["no-constant-patch"]` (and a
  reason) and pass it via `--config`; a waived file is silent.

**TypeScript** — parses each test file (`*.test.{ts,tsx,mts,cts}`) with the `oxc` parser and
walks the AST:

- **`no-first-party-mock`** — a `vi.mock()` / `vi.doMock()` whose target is a **first-party**
  module (a relative specifier like `./service` or `../core`). An integration test runs
  first-party code for real, so only third-party packages (`stripe`) and Node built-ins
  (`node:fs`, `child_process`) may be mocked. A non-literal target (`vi.mock(name)`) can't be
  classified deterministically and is left alone. See the [Isolation guide](../guide/isolation).

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
| `<PATH>`            | Root of the built artifact to inspect — an already-unpacked wheel, or a `dist/`.  |
| `--language <LANG>` | **Required.** `python` or `typescript`.                                           |

Scans `<PATH>` recursively for the language's test-file glob — `python` → `*_test.py`,
`typescript` → `*.test.*` — and exits `0` when none are present, `1` (printing each offending
path) when one is.

**Status (foundation):** the command scans an already-built artifact tree. The per-language
*build* step that produces that tree — wheel/sdist (Python), `dist` (TypeScript), `cargo
package` tarball (Rust, which also adds `--language rust`) — is landing per language.

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
