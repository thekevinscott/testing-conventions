---
description: Tune what the rules require in testing-conventions.toml — relax a coverage floor or exempt a file with a required reason.
---

# Configure the rules

This is the **what-to-enforce** surface: a single `testing-conventions.toml` at your repo root tunes
what the rules require — coverage floors and per-file exemptions. (For *where and how* a CI run is
scoped, that's the [workflow inputs](./ci); to make your *local* test runner match, see
[Extend the defaults](./extending).) Reach for this file when a strict default is wrong for your
project; anything you don't set keeps its default.

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
[Reference — Configuration](../reference/#configuration) for every key and
[Defaults](../reference/defaults) for the baseline.

## Exempt a file

Some files genuinely shouldn't be tested — a launcher shim, a re-export barrel, generated code. A
blocking gate needs that escape hatch, but here it's **explicit and reason-required**, never a
silent ignore.

### Empty files need no exemption

A file with no logic (empty, or only whitespace and comments) has nothing to test and is never
flagged — that's why a bare `__init__.py` needs no configuration, and why a TypeScript declaration
file (`*.d.ts`) is ignored. The moment a file gains a statement, it becomes a subject and needs
either a colocated test or an exemption.

### Exempt a real file

Add a `[[<language>.exempt]]` entry naming the rules it lifts and **why**:

```toml
# A launcher shim with no unit test:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["colocated-test"]
reason = "thin launcher; logic lives in run(), tested in run_test.py"

# Generated code you don't want in the coverage number:
[[python.exempt]]
path = "mypkg/pb/messages.py"
rules = ["coverage"]
reason = "generated protobuf stubs, not hand-authored"

# A re-export barrel, exempt from the colocated-test rule:
[[typescript.exempt]]
path = "src/index.ts"
rules = ["colocated-test"]
reason = "pure re-export barrel; no logic of its own"
```

- `path` is relative to the scanned `<PATH>`, and must point to a file that exists — a stale entry
  is a hard error, so the list can't silently rot.
- `rules` names the checks the entry lifts (`colocated-test`, `coverage`, a mutation or lint rule).
- `reason` is required; a reason-less entry is rejected when the config loads.

Because every exemption lives in this one file, names its rules, and carries a reason, the whole
exemption surface is auditable in a single diff — unlike scattered ignore comments. See
[Reference — Exemptions](../reference/#exemptions) for the exact schema.

### Narrow an exemption to specific lines

Exempting a whole file is a blunt instrument: one stubborn line — an equivalent mutant, a
cross-version import shim, a defensive branch that can't run on a single interpreter — forces you to
wave the *whole* module past the gate, testable code and all. For the **`coverage`** and
**`mutation`** rules, add a `lines` list to scope the exemption to exactly those lines:

```toml
[[python.exempt]]
path = "mypkg/config/tomlcompat.py"
rules = ["coverage", "mutation"]
lines = [9, 10, "12-13"]   # single lines and inclusive "start-end" ranges
reason = "version-conditional tomllib/tomli import; one branch is dead on any single interpreter"
```

The list is checked, not trusted — a **determinism guard** mirrors the stale-path rule:

- A listed line that **isn't actually failing** (it's covered, or has a killed mutant, or carries no
  measured code) is a **hard error** — you can't over-exempt.
- A line that **is** failing but **isn't listed** fails the gate as normal — you can't under-list
  and forget.

So the set is forced to be *exactly* the failing lines: minimal by construction, and as deterministic
as the rest of the standard. `lines` is **only** valid alongside `coverage` / `mutation` (the
measured-line rules); pairing it with `colocated-test` — a whole-file presence check — is rejected
on load. Omit `lines` and the entry stays whole-file, as before.

## See also

- [Reference — Configuration](../reference/#configuration): every key and the full schema.
- [Extend the defaults](./extending): reuse our shared test config locally.
- [The testing model — exemptions](../explanation/#exemptions-a-gate-needs-a-door): why a blocking gate needs a reasoned escape hatch.
