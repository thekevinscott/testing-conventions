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
ran but no test failed on), and exits non-zero if any survive.

| Language | Engine package | How it's provided |
| --- | --- | --- |
| TypeScript | [`@stryker-mutator/core`](https://stryker-mutator.io/) + `@stryker-mutator/vitest-runner` | Bundled (npm deps); resolved from the tool's own tree |
| Python | [`cosmic-ray`](https://github.com/sixty-north/cosmic-ray) | Bundled (wheel dep); resolved from the tool's env |
| Rust | [`cargo-mutants`](https://github.com/sourcefrog/cargo-mutants) | Installed separately — `cargo install cargo-mutants` ([#242](https://github.com/thekevinscott/testing-conventions/issues/242)) |

**You install no mutation engine** — it ships **bundled with `testing-conventions`** and the rule
resolves it from the tool's own install tree, never the runtime download path. For TypeScript the
engine (`@stryker-mutator/core` + its vitest runner) is co-located with the binary in the npm/npx
layout and resolved from there (or from your project, if you happen to pin your own); for Python the
wheel carries `cosmic-ray` in the same environment. You supply only the **test runner**
(`vitest` / `pytest`): it runs your own suite, so its version is yours.

> **Rust is the exception (for now).** `cargo-mutants` is a standalone binary, not a package
> dependency cargo can bundle, so the Rust arm still needs it installed (`cargo install cargo-mutants`,
> which the [CI workflow](./ci) does for you). Bringing Rust to the same install-nothing bar is tracked
> in [#242](https://github.com/thekevinscott/testing-conventions/issues/242).

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

`mutation` exemptions are line-scoped: `lines` names the survivor's line(s), and listing a line whose
mutants were all caught is a hard error. A passing run means every survivor was killed or explained.

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
