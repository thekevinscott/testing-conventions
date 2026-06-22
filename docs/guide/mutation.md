---
description: Run the unit mutation gate — standalone per language, diff-scoped with --base, and wired into the CI workflow.
---

# Run mutation testing

`unit mutation` is the verification rung above the coverage floor: it breaks the code and requires a
test to fail. (For *why* a binary gate instead of a score, see
[Why mutation testing](../explanation/mutation).) This guide runs it — standalone per language, and
in CI.

## Run it on a project

Point the rule at the project whose unit suite you want to verify, naming the language:

```sh
testing-conventions unit mutation --language rust .            # Rust
testing-conventions unit mutation --language typescript src/   # TypeScript
testing-conventions unit mutation --language python src/       # Python
```

The rule wraps each language's standard engine, collects every **surviving** mutant (one the suite
ran but no test failed on), and exits non-zero if any survive. Each engine must be installed:

| Language | Engine | Must be installed |
| --- | --- | --- |
| Rust | [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) | `cargo-mutants` |
| TypeScript | [Stryker](https://stryker-mutator.io/) | `@stryker-mutator/core` + a test-runner plugin |
| Python | [cosmic-ray](https://github.com/sixty-north/cosmic-ray) | `cosmic-ray` + `pytest` |

The gate is **on by default and binary**: any un-exempted survivor fails the run (exit `1`, listing
each survivor with its file, line, and mutation); a clean run exits `0`. There is no report-only mode
and config can't loosen it. See the [reference](../reference/#unit-mutation) for exit codes and the
exact engine outputs read.

## Scope it to a diff

Whole-tree mutation is slow, so scope the gate to the lines a change touched with `--base`:

```sh
testing-conventions unit mutation --language typescript --base origin/main src/
```

Only survivors on lines the `<base>...HEAD` diff added or modified count — *"no unexplained surviving
mutant on the lines you touched."* Each engine maps this to its own diff mode (cargo-mutants
`--in-diff`; Stryker `--mutate <file>:<line>-<line>` ranges; cosmic-ray changed-file scope + a
changed-line filter), all at line granularity.

## Exempt a survivor

A survivor you've confirmed is equivalent or deliberately defensive is lifted with a reason — the
same [exemption](./configure#exempt-a-file) mechanism every rule shares:

```toml
[[typescript.exempt]]
path = "src/clamp.ts"
rules = ["mutation"]
lines = [12]   # the exact line the surviving mutant is on — required for `mutation`
reason = "equivalent mutant: the `>= 0` guard can't be reached after the earlier abs()"
```

`mutation` exemptions are line-scoped: `lines` names the exact line(s) the survivor sits on, and a
listed line whose mutants were *all caught* is a hard error — so the list can only ever be the real
survivors. A passing run then means every survivor was either killed or explained.

## In CI

The [reusable workflow](./ci) runs `unit mutation` automatically — **on by default**, on pull
requests only, **diff-scoped** to the `<base>...HEAD` changed lines. A PR fails on any un-exempted
survivor on a changed line. It needs no configuration beyond the [drop-in](../getting-started);
`base` defaults to `origin/main`. All three languages are at parity, so the job fans out over each
language present.

## See also

- [Why mutation testing](../explanation/mutation) — the concept, equivalent mutants, and why it's a binary gate.
- [Reference — `unit mutation`](../reference/#unit-mutation) — flags, exit codes, and per-engine detail.
- [Configure the rules — exempt a file](./configure#exempt-a-file) — the reason-required escape hatch.
