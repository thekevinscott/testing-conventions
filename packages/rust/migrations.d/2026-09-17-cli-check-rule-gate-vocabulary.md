### Shipped CLI text converges on check / rule / gate

**Summary**

`docs/reference/glossary.md` ratified one meaning each for "check" (one of the eight runnable
units), "rule" (an assertion inside a lint check), and "gate" (a check in its merge-blocking role).
The docs and `README.md` converged on this already; the shipped binary's own `--help` text and
printed errors did not. `unit mutation`'s doc comment called itself "the gate", `e2e verify --base`
described the changed-line "coverage/mutation gates", the missing `--ts-mutation-adapter` error and
the two `unit coverage` vitest errors said "the rule", and two `config.rs` validation messages used
a bare, ambiguous "rules" for entries that can be either a check id or a rule id.

**Required changes**

_None._ Subcommand, flag, and config-key spelling are unchanged, including `rules = […]`.

**Deprecations removed**

_None._

**Behavior changes without code changes**

Only prose changed. Exit codes, output shape, and which strings appear are the same; the words
naming the eight units read as "check" instead of "gate" or "rule" where that was previously
inconsistent:

- `unit mutation --help`: "The gate is on by default" → "The check is on by default".
- `e2e verify --help`: "the changed-line coverage/mutation gates read the diff" → "…checks read
  the diff".
- `unit mutation --language typescript` without `--ts-mutation-adapter`: "run the rule through
  that CLI" → "run the check through that CLI".
- `unit coverage --language typescript` vitest errors: "The rule runs the project's own vitest" →
  "The check runs the project's own vitest".
- `testing-conventions.toml` validation: "whole-file exemptions are for presence / lint rules
  only" → "only `coverage` and `mutation` are line-scoped; every other check or rule is
  whole-file"; "move the whole-file rules to a separate entry" → "move the rest to a separate
  entry".

**Verification**

```console
$ testing-conventions unit mutation --help
...
Run mutation testing over the unit suite and fail on any surviving mutant not
lifted by a `mutation` exemption — the rung above coverage. The check is
on by default (no report-only mode). ...
```
