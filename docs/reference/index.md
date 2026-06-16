# Reference

Information-oriented reference for the `testing-conventions` CLI and its config file.

## CLI

```
testing-conventions <COMMAND>
```

Global flags: `--help`, `--version`.

### `unit colocated-test`

Check that every source file under a directory has a colocated unit test. `colocated-test` is
the first rule under the `unit` command group; future rules (e.g. `unit isolation`) and other
test kinds (`integration`, `e2e`) nest the same way.

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

Run the unit suite under coverage and fail if it's below the configured floor.

```
testing-conventions unit coverage --language <LANG> --config <CONFIG> <PATH>
```

| Argument / flag     | Description                                                                |
| ------------------- | -------------------------------------------------------------------------- |
| `<PATH>`            | Directory whose unit suite is run and measured.                            |
| `--language <LANG>` | **Required.** `python` only for now (TypeScript / Rust coverage are separate items). |
| `--config <CONFIG>` | Config file providing the thresholds (default `testing-conventions.toml`). |

For **`python`**, runs `coverage.py` with branch coverage on — measuring the sources under
`<PATH>` with `*_test.py` excluded from the denominator — and compares the total against
`[python].coverage` (`fail_under`, `branch`). Exits `0` when the floor is met, `1` (with the
actual vs. required percent on stderr) when it isn't. `coverage` and `pytest` must be installed.
Files with a `coverage` [exemption](#exemptions) are also excluded from the denominator.

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

Lint Python test files for mocking mechanism & style. The first rule under the `integration`
command group; future lints join it under the same command.

```
testing-conventions integration lint --language <LANG> <PATH>
```

| Argument / flag     | Description                                                        |
| ------------------- | ----------------------------------------------------------------- |
| `<PATH>`            | Directory to scan recursively for Python test files.              |
| `--language <LANG>` | **Required.** `python` only for now. Omitting it is a usage error. |

Parses each Python test file (`*_test.py`, `test_*.py`, `conftest.py`) with a Rust Python
parser and walks the AST. Reports each violation to stderr as `path:line: <lint> — <message>`
and exits `1` if any are found, `0` otherwise.

**Lints:**

- **`no-monkeypatch`** — a test or fixture function that declares the `monkeypatch` parameter.
  pytest's `monkeypatch` is banned; patch with `unittest.mock` (`patch` / `patch.object` /
  `patch.dict`) wrapped in a `pytest.fixture` instead.
- **`no-inline-patch`** — a `patch(...)` / `patch.object(...)` / `patch.dict(...)` call in a
  test body, whether the `with patch(...)` form or a bare call. Move the patch into a
  `pytest.fixture`; a patch inside a fixture is allowed.

### `check`

Reserved for the config-driven umbrella that runs every configured rule. **Not wired yet** —
it currently exits `0`. Rules ship under their test-kind group (like `unit colocated-test`)
until `check` orchestrates them from the config.

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
