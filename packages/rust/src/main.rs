// Declaration-only by design: Rust requires a `main` in the binary root, and a `fn main` with a
// body is a unit-gate subject. Re-exporting carries no function, so the logic lives in
// `entrypoint`, under test. See docs/reference/checks/colocated-test.md.
pub use testing_conventions::entrypoint::main;
