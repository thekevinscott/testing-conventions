**BREAKING** Rust `unit mutation` restricts the judging suite to `--cargo-test-arg --lib`,
matching `unit coverage`'s own `--lib`. It previously handed cargo-mutants no target restriction,
so cargo-mutants fell back to its default `cargo test`, building and running the crate's whole
`tests/` tree once per mutant — mutation and coverage measured different slices, and a crate whose
`tests/` held an e2e tier ran it in CI. A mutant only the integration or e2e tier caught now
survives. See `../migrations.d/2026-09-17-rust-mutation-scoped-to-lib.md`.
