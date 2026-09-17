**Changed** Shipped `--help` text and printed errors now use "check" where they meant one of the
eight runnable units, matching `docs/reference/glossary.md`. `unit mutation --help`, `e2e verify
--help`, the missing `--ts-mutation-adapter` error, and the two `unit coverage` vitest errors said
"gate" or "rule" for what is a check; two `config.rs` validation errors dropped an ambiguous bare
"rules". No config key, CLI subcommand, or flag name changed, and `rules = […]` keeps its spelling.
See `../migrations.d/2026-09-17-cli-check-rule-gate-vocabulary.md`.
