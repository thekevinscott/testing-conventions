//! E2E tests for the TypeScript packaging slice: drive the built CLI binary
//! against pre-built `npm pack` **tarball** fixtures (`.tgz`) and assert the exit
//! code. Rule (README "Packaging"): test files must never ship in the built
//! artifact — here the published npm package. `red.tgz` ships
//! `package/dist/widget.test.js`; `clean.tgz` does not.
//!
//! These start red — the foundation unpacks a `.whl` (zip) or a
//! directory, not a `.tgz` (gzipped tar) — and go green once `packaging` unpacks
//! the tarball before scanning.

use std::path::PathBuf;
use std::process::Command;

fn tarball(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging/typescript_npm")
        .join(name)
}

/// Exit code of `testing-conventions packaging <tarball> --language typescript`.
fn packaging_exit(artifact: &str) -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("packaging")
        .arg(tarball(artifact))
        .args(["--language", "typescript"])
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn a_tarball_shipping_a_test_file_exits_nonzero() {
    assert_eq!(packaging_exit("red.tgz"), 1);
}

#[test]
fn a_clean_tarball_exits_zero() {
    assert_eq!(packaging_exit("clean.tgz"), 0);
}
