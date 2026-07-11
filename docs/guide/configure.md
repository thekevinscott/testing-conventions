---
description: Respond to a red check in testing-conventions.toml — relax a coverage floor or exempt a file or line with a required reason.
---

# Configure the rules

When a check goes red, there are two responses: fix the code, or record a deliberate omission
here. A single `testing-conventions.toml` at your repo root (or per package, in a
[monorepo](../monorepo)) tunes what the rules require — coverage floors and reason-required
exemptions. Anything you don't set keeps its strict default; the
[configuration reference](../reference/config) carries every key.

## Relax a coverage floor

Every rule ships a strict 100% floor. Lower one under the language's table:

```toml
# Drop the Python floor from the strict default 100 to 90:
[python]
coverage = { branch = true, fail_under = 90 }

[typescript]
coverage = { branches = 90 }   # lines / functions / statements stay at 100

[rust]
coverage = { regions = 90 }    # lines stays at 100
```

A `[<language>].coverage` table is a **partial override** — set only the fields you want to move and
the rest keep their default. (A typo'd key is still rejected; only *missing* keys fall back.) See
[Reference — coverage keys](../reference/config#coverage) for every key and default, and
[Why a 100% floor](../explanation/coverage) before you lower one.

## Exempt a file

Some files genuinely shouldn't be tested — a launcher shim, a re-export barrel, generated code. A
blocking gate needs that escape hatch, but here it's **explicit and reason-required**, never a
silent ignore — see [Scoping and exemptions](../explanation/scoping) for the design.

### Empty files need no exemption

A file with no logic (empty, or only whitespace and comments) has nothing to test and is never
flagged — that's why a bare `__init__.py` needs no configuration, and why a TypeScript declaration
file (`*.d.ts`) is ignored. The moment a file gains a statement, it becomes a subject and needs
either a colocated test or an exemption.

### Exempt a real file

Add a `[[<language>.exempt]]` entry naming the rules it lifts and **why**. Whole-file exemptions are
for the **presence and lint** rules — a launcher shim with no colocated test, a re-export barrel with
no logic to isolate:

```toml
# A launcher shim with no unit test:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test"]
reason = "thin launcher; logic lives in run(), tested in run_test.py"

# A re-export barrel, exempt from the colocated-test rule:
[[typescript.exempt]]
path = "src/index.ts"
rules = ["colocated-test"]
reason = "pure re-export barrel; no logic of its own"
```

- `path` is relative to the scanned `source`, and must point to a file that exists — a stale entry
  is a hard error, so the list can't silently rot.
- `rules` names the checks the entry lifts (`colocated-test`, a mutation or lint rule). For
  `coverage` / `mutation`, see the line-scoped form below — those are never whole-file.
- `reason` is required; a reason-less entry is rejected when the config loads.

Because every exemption lives in this one file, names its rules, and carries a reason, the whole
exemption surface is auditable in a single diff — unlike scattered ignore comments. See
[Reference — Exemptions](../reference/config#exemptions) for the exact schema.

### Exempt specific lines (`coverage` / `mutation`)

`coverage` and `mutation` exemptions are never whole-file — they must carry a `lines` list naming the
exact lines they lift:

```toml
[[python.exempt]]
path = "mypkg/config/tomlcompat.py"
rules = ["coverage", "mutation"]
lines = [9, 10, "12-13"]   # single lines and inclusive "start-end" ranges
reason = "version-conditional tomllib/tomli import; one branch is dead on any single interpreter"
```

A **determinism guard** checks the list:

- A listed line that **isn't failing** (it's covered, has a killed mutant, or carries no measured
  code) is a **hard error**.
- A failing line that **isn't listed** fails the gate as normal.

So the list is exactly the failing lines. `lines` is required with `coverage` / `mutation` and
rejected with any whole-file rule, so the two never share an entry — a file exempt from both
`colocated-test` and `coverage` is two entries.

## See also

- [Reference — Configuration](../reference/config): every key and the full schema.
- [Scoping and exemptions](../explanation/scoping): why a blocking gate needs a reasoned door.
