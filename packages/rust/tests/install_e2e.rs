//! E2E tests for `install`: drive the built CLI binary with
//! no mocks and assert the block it writes points only at pages that exist —
//! the docs site root and the machine-readable contract (`llms.txt`), with the
//! removed CLI guide page unlinked.
//!
//! Starts red against the template that still links the removed `guide/cli`
//! page and goes green once the template drops it.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway working directory, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-install-e2e-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&root).unwrap();
        TempDir(root)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn install_cli_writes_the_block_with_live_pointers_only() {
    let dir = TempDir::new();

    let status = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("install")
        .current_dir(&dir.0)
        .status()
        .expect("the built binary should run");
    assert!(status.success(), "install should exit 0");

    let text = fs::read_to_string(dir.0.join("AGENTS.md")).expect("install writes AGENTS.md");
    assert!(
        text.contains("https://thekevinscott.github.io/testing-conventions/llms.txt"),
        "the block links the machine-readable contract"
    );
    assert!(
        !text.contains("https://thekevinscott.github.io/testing-conventions/guide/cli"),
        "the block must not link the removed CLI guide page (#353)"
    );
}
