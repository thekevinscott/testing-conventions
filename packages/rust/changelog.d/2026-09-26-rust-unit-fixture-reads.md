**Changed** `no-out-of-module-call` and `no-out-of-module-import` allow `std::fs` and
`std::env::temp_dir` in a Rust unit test (#708). Rust privacy makes the inline `#[cfg(test)]`
module the only tier that can reach a private item, so a private path-walker can be tested
nowhere else — and its argument is a directory that has to exist. The rest of `std::env` (`var`,
`set_var`, `args`) is still flagged: it reads ambient state the test never created. This is a
deliberate asymmetry with the Python and TypeScript unit rules, where a sibling test file reaches
module-private code and filesystem work belongs in the integration tier. See
[`../migrations.d/2026-09-26-rust-unit-fixture-reads.md`](../migrations.d/2026-09-26-rust-unit-fixture-reads.md).
