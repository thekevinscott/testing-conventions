**Fixed** Rust `mutation` no longer reports a survivor for a mutant inside an item a
`#[cfg(not(test))]` gate keeps out of the test build. The unit tier runs `--lib --bins`, which sets
`cfg(test)`, so that code is absent from the binary the suite runs and no test can kill its mutants;
cargo-mutants mutates it anyway because it reads the source, not the build. The exclusion is
item-level, so a gated entry point beside a tested function in the same file drops out on its own,
and it covers only a gate a test build genuinely cannot satisfy — `#[cfg(not(test))]` and
`#[cfg(all(not(test), unix))]` qualify, while `#[cfg(any(not(test), unix))]` still compiles under
`cargo test` and stays a subject.
