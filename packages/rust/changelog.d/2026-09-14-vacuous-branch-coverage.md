**Fixed** Python `unit coverage` no longer fails a source with zero branch points on the
"branch coverage is required but the report measured no branches" guard. The guard's purpose is
to catch a run where branch measurement was never enabled — but after #635 scoped the run with
`--source`, a fully covered straight-line package (13 statements, no `if`/`for`/`and`/`or`)
measures zero branches *with* `--branch` on, and 100% of nothing failed as misconfigured. The
gate now reads coverage.py's `meta.branch_coverage` flag, which the report sets exactly when
branch measurement was active, and treats "measured, none exist" as a vacuously met floor.
