**Fixed** Python `unit coverage` no longer collects `tests/integration/` or `tests/e2e/` into the
unit run when the scanned `source` is itself the package root. The pytest invocation passed `.` as
its collection root — the same directory `coverage`'s `--source` already scoped for measurement —
so a package with no separate `source` nesting ran the integration suite under the unit floor too,
inheriting its environment preconditions and letting integration-only coverage count toward a unit
gate. `pytest` now ignores a `tests/` directory directly beneath the scanned path, matching how
`cargo llvm-cov --lib` and vitest's default include glob already keep those tiers out of the Rust
and TypeScript unit runs. See
`../migrations.d/2026-09-17-python-unit-coverage-ignores-suite-tiers.md`.
