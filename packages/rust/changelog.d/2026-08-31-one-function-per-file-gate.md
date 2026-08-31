**Added** **The reusable workflow runs `unit one-function-per-file`** (#512). The rule shipped as
a CLI subcommand with its workflow wiring left for a follow-up, so a consumer who adopted
`conventions.yml` whole-hog got every gate except this one and could reach the rule only by
invoking the binary by hand. It now runs as a fifth step of the `Static checks (<language>)` job,
over the same rust-inclusive language set the other static gates use, and answers to
`one-function-per-file` in the `gates` allowlist. `detect` gained a `one_function_languages`
output to drive it.

Python and TypeScript are checked at the shipped default of `max_lines = 1`; Rust reports that the
rule is not enabled and passes until a `[rust].one_function_per_file` table opts in. A tree
adopting the gate onto existing source declares the threshold that passes today in its
`testing-conventions.toml` and walks it down. See
[`../migrations.d/2026-08-31-one-function-per-file-gate.md`](../migrations.d/2026-08-31-one-function-per-file-gate.md).
