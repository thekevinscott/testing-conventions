**BREAKING** **Removed** the `check` subcommand. It was a scaffold: `--help` listed it, it parsed,
and it exited `0` without running a rule, so a consumer who called it got a pass nothing had
earned. It now exits non-zero with `unrecognized subcommand`. A bare `testing-conventions` with no
subcommand is unchanged — banner on stderr, exit `0`. See
`../migrations.d/2026-08-31-remove-check-subcommand.md`.
