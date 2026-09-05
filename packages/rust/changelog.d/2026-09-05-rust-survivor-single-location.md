**Fixed** A Rust mutation survivor line carries one location. cargo-mutants' mutant name embeds
its own `file:line:col:` prefix, so the line read
`src/lib.rs:7: src/lib.rs:7:5: replace > with == in is_positive`; the embedded prefix is now
stripped and the line reads `src/lib.rs:7: replace > with == in is_positive`, matching the shape
of the Python and TypeScript lines. See
[`../migrations.d/2026-09-05-rust-survivor-single-location.md`](../migrations.d/2026-09-05-rust-survivor-single-location.md).
