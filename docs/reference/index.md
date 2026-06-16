# Reference

Information-oriented reference for the `testing-conventions` CLI and its config file.

## CLI

```
testing-conventions <COMMAND>
```

Global flags: `--help`, `--version`.

### `unit location`

Check that every source file under a directory has a colocated unit test. `location` is the
first rule under the `unit` command group; future rules (e.g. `unit isolation`) and other
test kinds (`integration`, `e2e`) nest the same way.

```
testing-conventions unit location --language <LANG> <PATH>
```

| Argument / flag     | Description                                                       |
| ------------------- | ----------------------------------------------------------------- |
| `<PATH>`            | Directory to scan recursively.                                    |
| `--language <LANG>` | **Required.** Convention to enforce: `python` or `typescript`. No default — omitting it is a usage error, never a silent `python` run. |

**What counts, by language:**

- **`python`** — a source `*.py` needs a colocated `*_test.py`. `*_test.py` files (the tests
  themselves) and `__init__.py` (a language-mandated package marker) are not subjects.
- **`typescript`** — a source `*.ts` / `*.tsx` / `*.mts` / `*.cts` needs a colocated
  `*.test.*` of the matching extension (`foo.mts` → `foo.test.mts`). `*.test.*` files are the
  tests; declaration files (`*.d.ts` / `*.d.mts` / `*.d.cts`) carry no runtime code and are
  ignored. Nothing else is exempt.

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

### `check`

Reserved for the config-driven umbrella that runs every configured rule. **Not wired yet** —
it currently exits `0`. Rules ship under their test-kind group (like `unit location`) until
`check` orchestrates them from the config.

## Configuration

The standard is config-driven: one TOML file is the single source of truth for every rule's
thresholds. The schema is validated by the loader (unknown keys and malformed TOML are
rejected). The `[python].coverage` thresholds are consumed by `unit coverage` today; the
other tables are accepted but not yet enforced (their rules are forthcoming).

```toml
[python]
coverage = { branch = true, fail_under = 100 }

[typescript]
coverage = { lines = 100, branches = 100, functions = 100, statements = 100 }

[rust]
coverage = { regions = 100, lines = 100 }
```

Each top-level table (`[python]`, `[typescript]`, `[rust]`) is optional. See
[Migrations](../migrations) for the public-API history.
