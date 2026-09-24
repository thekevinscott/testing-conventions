**BREAKING** Rust `unit coverage` and `unit mutation` now measure the crate's binary targets
alongside its library: both pass `--bins` next to the existing `--lib`. `colocated-test` requires an
inline `#[cfg(test)]` module in every source file holding testable code, binary sources included,
but `--lib` alone never built or ran those modules — the gate demanded tests the runs then ignored.
A binary target's colocated tests now execute, so its uncovered lines count against the coverage
floor and its surviving mutants fail the gate.

Rust `colocated-test` also stops treating a logic-free entry point as a subject: a `fn main` whose
body is one argument-free call and nothing else reads as a declaration, so a binary root that
delegates into the library needs no inline test module and no exemption. Anything wider — a second
statement, an argument, an operator, a `?`, a `match`, a method call — stays a subject. See
`../migrations.d/2026-09-24-rust-binary-target-unit-suite.md`.
