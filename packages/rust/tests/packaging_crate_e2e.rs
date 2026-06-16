//! E2E tests for the Rust packaging slice (#74): drive the built CLI binary
//! against pre-built `cargo package` **crate tarball** fixtures (`.crate`) and
//! assert the exit code. Rule (README "Packaging"): inline `#[cfg(test)]` units
//! compile out of the consumer artifact for free, so the source tarball must not
//! ship the crate-root `tests/` directory. `widget-0.1.0.crate` leaks
//! `tests/integration.rs`; `clean-0.1.0.crate` (a crate with
//! `exclude = ["tests/**"]`) does not.
//!
//! These start red: `--language rust` isn't a value the `packaging` command
//! accepts yet (so the binary errors at argument parsing), and `.crate` isn't a
//! recognized archive. They go green once #74 adds the Rust language plus the
//! `.crate` / `tests/`-directory handling.

use std::path::PathBuf;
use std::process::Command;

fn crate_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/rust_crate")
        .join(name)
}

/// Exit code of `testing-conventions packaging <crate> --language rust`.
fn packaging_exit(artifact: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("packaging")
        .arg(crate_fixture(artifact))
        .args(["--language", "rust"])
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn a_crate_shipping_the_tests_dir_exits_nonzero() {
    assert_eq!(packaging_exit("widget-0.1.0.crate"), 1);
}

#[test]
fn a_clean_crate_exits_zero() {
    assert_eq!(packaging_exit("clean-0.1.0.crate"), 0);
}
