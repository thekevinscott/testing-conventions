# `internals/checks` — the repo-only self-test checks package

`tc-checks` is one `@click.group()` (`checks/cli.py`); each check is a `@click.command()` in its
own subpackage, registered on the group with one `add_command` line. These are the structural
conventions every check follows — hold to them so a new check is a copy of an existing one, not a
new shape.

## One function per file

A pure function that is **more than a few lines** lives in **its own module** beside the command,
with its **own colocated `*_test.py`**. `checks/<check>/cli.py` holds only the thin
`@click.command()` — it imports the pure function and wires it to `CheckFailed` / `click.echo`.

- `checks/<check>/decide.py` (or a name that fits) → the decision function + `decide_test.py`.
- `checks/<check>/cli.py` → the command + `cli_test.py` (drives `.callback`, asserts `cli.params`).

A genuine one-liner predicate does **not** earn a function or a file — inline it (see below).

## Inline the trivial

Do not wrap a single expression in a named helper or hoist it into a module global:

- **A one-substring check** goes straight in the command: `if "marker" not in Path(workflow).read_text(): raise CheckFailed(...)`. No `wires_x()` function, no `MARKER = "…"` constant.
- **A single-use regex / value** is inlined at its use site: `re.search(r"…", text)`, not a module-level `_PATTERN = re.compile(…)` referenced once.

The command's own tests (a wired and an unwired fixture through `.callback`, plus the `cli.params`
metadata assertion) cover both branches and kill the string / operator / decorator mutants without
the extra layer.

## Shared constants live in `config.py`

A value more than one check needs — a workflow path, an expected detect output — goes in
`checks/config.py`, imported where used. Never copy the literal into each check. A check's colocated
test imports the constant **through its own `cli` module** (`from checks.<check>.cli import
REUSABLE_WORKFLOW`), so the isolation lint reads it as the unit under test, not a collaborator.

`config.py` is constants-only, so it carries no colocated test — it is exempted in
`testing-conventions.toml` (`colocated-test`, and `mutation` on the constant lines, since a mutated
path is observable only through the out-of-scope e2e file read).

## DRY the boilerplate into `utils/`

When a group of checks share machinery — a subprocess runner, an exit-code decision, an
orchestration loop — that machinery is **one shared module** under `checks/utils/`, unit-tested
once, not copied into every check. A check then reduces to its own data (the list of commands it
runs, the marker it looks for) plus a thin command that calls the shared runner. If you are about to
paste the same function into a second check, stop and lift it to `utils/` first.

## Tests

- Colocated `cli_test.py` imports **only the unit under test** — never `click`, `click.testing`
  (`CliRunner`), or `CheckFailed` (all flagged as collaborators). Drive the command through
  `cli.callback(...)`; assert the raise path against the propagated exception's `.message`.
- The `cli.params` metadata assertion (`argument.name` / `argument.default` / `nargs`) is what kills
  the `@click.argument` / `@click.option` RemoveDecorator mutant that `.callback` can't reach — keep
  it on every command.
- The full e2e suite (`CliRunner` over real files) lives at `internals/checks/tests/e2e/`, a sibling
  **outside** the scanned `src/`. A `*_test.py` e2e file inside `src/` would be read as an
  un-isolated unit test.

## No underscore-prefixed module names

An empty `__init__.py` already says "nothing public here"; modules are named plainly
(`decide.py`, not `_decide.py`).
