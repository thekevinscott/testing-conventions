---
description: Put the testing contract in your coding agent's context before it writes code — a managed, hash-versioned block in AGENTS.md, installed and kept fresh by the CLI.
---

# Surface the contract to your agent

Coding agents (Claude Code, Codex, Cursor, Gemini CLI, Copilot) read your repository's `AGENTS.md`
at session start. `testing-conventions agents install` writes a small managed block into that file,
so your agent knows the contract — red-first tests, colocated units, coverage and mutation floors,
cross-language parity, reasoned exemptions — **before** it writes code, rather than discovering the
rules one CI failure at a time.

The block is thin by design: the few non-negotiables plus pointers to the full contract and the
CLI. Your prose around it stays yours — the tool touches only the region between its own markers.

## Install the block

Run from the repository root:

```sh
testing-conventions agents install
```

- **No `AGENTS.md` yet** → the file is created containing the block.
- **`AGENTS.md` exists without the markers** → the block is appended; existing content is preserved
  byte for byte.
- **The markers are present** → only the region between them is replaced.

Re-running is idempotent: when the installed block already matches the current template, the file
is untouched and the command prints `current`. Pass a path to manage a different file:

```sh
testing-conventions agents install docs/AGENTS.md
```

The block is delimited by an HTML-comment marker pair. The begin marker carries the block's schema
version and a content hash, which is how staleness is detected later:

```md
<!-- testing-conventions:begin v1 hash=3c9egh12ab34 -->
…the managed contract block…
<!-- testing-conventions:end -->
```

## Keep it fresh

The template ships with the CLI, so upgrading the package can bring a newer block. `agents check`
compares the managed region against the current template and reports one word on stdout:

```sh
testing-conventions agents check
```

| Output    | Exit code | Meaning                                                        |
| --------- | --------- | -------------------------------------------------------------- |
| `current` | `0`       | The managed region matches the current template.               |
| `stale`   | `1`       | The region differs — an upgrade or a hand-edit changed it.     |
| `absent`  | `1`       | The file or the markers are missing.                           |

`check` is read-only, which makes it a CI nudge: add it as a step and a stale block fails the build
until someone re-runs `agents install`.

## Remove the block

```sh
testing-conventions agents remove
```

Deletes the managed region, markers included, and leaves the rest of the file exactly as it was.
Removal is idempotent: with no block (or no file) it prints `absent` and exits `0`.

## Safety

- **Only the marked region is ever written.** Content outside the markers survives every
  `install` / `remove` byte for byte.
- **Writes are atomic** — a temp file in the same directory, then a rename — so a crash leaves the
  original file intact.
- **Symlinks are refused.** When the target path is a symlink, `install` and `remove` warn on
  stderr and exit `1` with the file untouched.

## See also

- [Reference — `agents install` / `agents check` / `agents remove`](../reference/#agents-install):
  every flag and exit code.
- [The testing model](../explanation/): the contract the block points your agent at.
