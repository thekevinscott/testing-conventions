# Reference

Information-oriented reference for the `testing-conventions` CLI and its config file.

## CLI

```
testing-conventions <COMMAND>
```

Global flags: `--help`, `--version`.

### `unit-location`

Check that every source file under a directory has a colocated unit test.

```
testing-conventions unit-location [--lang <LANG>] <PATH>
```

| Argument / flag | Description                                                        |
| --------------- | ------------------------------------------------------------------ |
| `<PATH>`        | Directory to scan recursively.                                     |
| `--lang <LANG>` | Convention to enforce: `python` (default) or `typescript`.         |

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

### `check`

Reserved for the config-driven umbrella that runs every configured rule. **Not wired yet** —
it currently exits `0`. Rules ship as their own subcommand (like `unit-location`) until
`check` orchestrates them from the config.

## Configuration

The standard is config-driven: one TOML file is intended as the single source of truth for
every rule's thresholds. The schema is validated by the loader (unknown keys and malformed
TOML are rejected), but **no rule consumes it from the CLI yet** — the rule that ships today
(`unit-location`) is deliberately not configurable, and the coverage thresholds below are
accepted by the schema ahead of the coverage engine that will enforce them.

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
