**Changed** `unit one-function-per-file` prints a one-line summary to stderr on a passing run —
`one-function-per-file: scanned N file(s), 0 violations` — alongside the version banner. A real
pass and a vacuous run over an empty or wrongly-scoped tree previously produced byte-identical
output; the scanned count now tells them apart. See
`../migrations.d/2026-09-17-one-function-per-file-pass-summary.md`.
