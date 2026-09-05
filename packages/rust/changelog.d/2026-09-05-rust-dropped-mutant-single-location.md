**Fixed** The Rust zero-mutant proof lists each dropped mutant with one location. cargo-mutants'
mutant name embeds its own `file:line:col:` prefix, so a site read
`src/lib.rs:7: src/lib.rs:7:7: replace > with == in is_positive`; the embedded prefix is now
stripped and the site reads `src/lib.rs:7: replace > with == in is_positive`, matching the
survivor line. A name carrying no embedded prefix still renders its location. See
[`../migrations.d/2026-09-05-rust-dropped-mutant-single-location.md`](../migrations.d/2026-09-05-rust-dropped-mutant-single-location.md).
