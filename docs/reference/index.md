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
  ignored, as are pure re-export **barrels** (see [Exemptions & waivers](#exemptions-waivers)).

Beyond the per-language rules, a file can be exempted by a structural **barrel** match or an
explicit **waiver** — see [Exemptions & waivers](#exemptions-waivers).

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
Files carrying a `coverage` [waiver](#exemptions-waivers) are also excluded from the
denominator.

## Exemptions & waivers

Not every source file should need a colocated test or full coverage — a launcher shim, a pure
re-export barrel, generated code. So the checker can be a *blocking* gate without forcing
pointless tests, two escape hatches exist. Neither is a silent blanket ignore: structural
exemptions are deterministic, and waivers are reason-required and visible in the file's own
diff.

### Structural exemptions (automatic)

Matched by shape, no configuration. A file that matches is never an orphan:

| Exemption | Language | Why |
| --------- | -------- | --- |
| `__init__.py` | Python | Language-mandated package marker; no runtime logic. |
| `*.d.ts` / `*.d.mts` / `*.d.cts` | TypeScript | Declaration files carry no runtime code. |
| Pure re-export **barrel** | TypeScript | A file whose only statements are `export … from "…"` (e.g. `index.ts`, `public-api.ts`). Wires modules together but holds no logic of its own — the TypeScript analog of `__init__.py`. Matched by **shape, not name**. |

A barrel is recognized when *every* statement is a re-export — `export * from`,
`export * as ns from`, `export { … } from`, with an optional `type` modifier. A single local
declaration (`export const`, `export function`, an `import`) makes the file a subject again.

### Waivers (explicit, reason-required)

For deliberate omissions the tool can't infer, add an in-file marker — a comment, anywhere in
the file:

```
testing-conventions:waiver(<scope>): <reason>
```

| Field | Values |
| ----- | ------ |
| `<scope>` | `location` (skip the colocated-test requirement), `coverage` (omit from the coverage denominator), or `all` (both). |
| `<reason>` | Free text to the end of the line. **Required** — a marker with an empty reason is an error. |

```ts
// testing-conventions:waiver(location): thin CLI launcher; logic lives in run(), tested in run.test.ts
export const main = () => process.exit(cli(process.argv));
```

```python
# testing-conventions:waiver(coverage): generated protobuf stubs, not hand-authored
```

The marker token `testing-conventions:waiver` is **reserved**: an occurrence that isn't a
valid `(scope): reason` makes the check **error**, so a typo can't quietly disable it. Because
the waiver lives at the omission and carries a reason, it shows up in code review and the
file's diff — auditable by construction, the opposite of a prose omit-list.

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
