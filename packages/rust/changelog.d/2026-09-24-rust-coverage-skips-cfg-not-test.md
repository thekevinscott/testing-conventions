**Fixed** Rust `coverage` no longer counts an item a `#[cfg(not(test))]` gate keeps out of the test
build, whole-tree or changed-line. The unit tier runs `cargo llvm-cov --lib --bins`, which sets
`cfg(test)`, so no test can execute the item; the `--bins` half links the library a second time as a
plain dependency of the binary target's test harness, where `cfg(test)` is unset and the item is
compiled and instrumented as 0-hit. Whether those counters reached the report depended on how the
linker partitioned the build, so the same source measured differently on two machines. The exclusion
is item-level and covers only a gate a test build genuinely cannot satisfy — `#[cfg(not(test))]` and
`#[cfg(all(not(test), unix))]` qualify, while `#[cfg(any(not(test), unix))]` still compiles under
`cargo test` and stays a subject.
