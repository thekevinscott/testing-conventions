# Exempt a file you can't (or shouldn't) test

A blocking gate is only honest if it has an escape hatch for the files that genuinely
shouldn't be tested — a launcher shim, a pure re-export barrel, generated code. Otherwise the
checker forces pointless tests or fights your conventions, and someone disables it.

`testing-conventions` keeps that escape hatch **explicit and in one place**: the config file.
There's no automatic name- or shape-based exemption to reason about — the only files skipped
automatically are the ones with no code at all.

## Empty files need nothing

A file with no logic — empty, or only whitespace and comments — has nothing to unit-test and is
never flagged. That's why a bare `__init__.py` needs no configuration, and why a TypeScript
declaration file (`*.d.ts`) is ignored. The moment a file gains a statement (a re-export, a
constant, a function), it becomes a subject and needs either a colocated test or an exemption.

## Exempt a real file in config

For a deliberate omission, add a `[[<language>.exempt]]` entry naming the rules it lifts and
**why**:

```toml
# testing-conventions.toml

# A launcher shim with no unit test:
[[python.exempt]]
path = "mypkg/cli.py"
rules = ["location"]
reason = "thin launcher; logic lives in run(), tested in run_test.py"

# Generated code you don't want in the coverage number:
[[python.exempt]]
path = "mypkg/pb/messages.py"
rules = ["coverage"]
reason = "generated protobuf stubs, not hand-authored"

# A re-export barrel — exempt from the colocated-test rule:
[[typescript.exempt]]
path = "src/index.ts"
rules = ["location"]
reason = "pure re-export barrel; no logic of its own"
```

- `path` is relative to the scanned `<PATH>`.
- `rules` is `location` (skip the colocated-test requirement), `coverage` (omit from the
  coverage denominator), or both.
- `reason` is required — a reason-less entry is rejected when the config loads.

`unit location` reads the list via `--config` (default `testing-conventions.toml`); `unit
coverage` already takes `--config` for its thresholds.

## Why config, not in-file comments

Keeping exemptions in the one config file means the project's **entire** exemption surface is
visible in a single file — and in a single diff when it changes. A reviewer sees every
deliberate omission at once, instead of hunting for ignore-comments scattered across the
source.

It's enforced, too: every `path` must point to a file that exists. A stale entry — left behind
when a file moves or is deleted — is a **hard error**, not a silently-ignored line:

```sh
$ testing-conventions unit location --language python src/
error: exempt entry `mypkg/cli.py` matches no file under `src` — remove the stale entry or fix the path
```

That's the difference between an auditable exemption and an ignore-glob that quietly rots.

## See also

- [Reference — Exemptions](../reference/#exemptions) — the exact schema and the empty-file rule.
