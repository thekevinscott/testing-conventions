//! Integration tests for `install`'s managed block content.
//!
//! The docs site reorganized around workflow adoption and removed the
//! CLI guide page, so the block `install` writes must point only at pages that
//! exist: the docs site root and the machine-readable contract (`llms.txt`).
//! These start red against the template that still links the removed
//! `guide/cli` page and go green once the template drops it.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::agents::install;

/// A throwaway directory for the managed file, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-install-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&root).unwrap();
        TempDir(root)
    }

    fn agents_md(&self) -> PathBuf {
        self.0.join("AGENTS.md")
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const DOCS_ROOT: &str = "https://thekevinscott.github.io/testing-conventions/";
const CONTRACT: &str = "https://thekevinscott.github.io/testing-conventions/llms.txt";
const REMOVED_CLI_PAGE: &str = "https://thekevinscott.github.io/testing-conventions/guide/cli";

#[test]
fn install_points_at_the_docs_root_and_the_machine_readable_contract() {
    let dir = TempDir::new();

    install(&dir.agents_md()).expect("install should succeed");

    let text = fs::read_to_string(dir.agents_md()).unwrap();
    assert!(text.contains(DOCS_ROOT), "the block links the docs site");
    assert!(
        text.contains(CONTRACT),
        "the block links the machine-readable contract"
    );
    assert!(
        !text.contains(REMOVED_CLI_PAGE),
        "the block must not link the removed CLI guide page (#353)"
    );
}

#[test]
fn reinstall_replaces_a_stale_block_carrying_the_removed_link() {
    // A consumer whose AGENTS.md holds a block written before the docs
    // reorganization: re-running `install` refreshes the owned region to the
    // current pointers and touches nothing outside the markers.
    let dir = TempDir::new();
    let stale = format!(
        "# My project\n\nHouse rules stay.\n\n\
         <!-- testing-conventions:begin v1 hash=000000000000 -->\n\
         ## Testing conventions\n\n\
         Run the rules locally with the CLI: {REMOVED_CLI_PAGE}\n\
         <!-- testing-conventions:end -->\n"
    );
    fs::write(dir.agents_md(), &stale).unwrap();

    install(&dir.agents_md()).expect("install should succeed");

    let text = fs::read_to_string(dir.agents_md()).unwrap();
    assert!(
        text.starts_with("# My project\n\nHouse rules stay.\n\n"),
        "content outside the markers is untouched"
    );
    assert!(
        text.contains(CONTRACT),
        "the refreshed block links the machine-readable contract"
    );
    assert!(
        !text.contains(REMOVED_CLI_PAGE),
        "a refresh drops the removed CLI guide link (#353)"
    );
}
