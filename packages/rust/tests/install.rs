use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::agents::install;

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
        "the block must not link the removed CLI guide page"
    );
}

#[test]
fn reinstall_replaces_a_stale_block_carrying_the_removed_link() {
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
        "a refresh drops the removed CLI guide link"
    );
}

#[test]
fn install_appends_after_existing_prose_without_a_marker() {
    let dir = TempDir::new();
    fs::write(dir.agents_md(), "# My project\n\nHouse rules stay.").unwrap();

    install(&dir.agents_md()).expect("install should succeed");

    let text = fs::read_to_string(dir.agents_md()).unwrap();
    assert!(
        text.starts_with("# My project\n\nHouse rules stay.\n\n<!-- testing-conventions:begin "),
        "the block lands after the prose, separated by a blank line; got: {text}"
    );
    assert!(text.contains(CONTRACT));
}

#[test]
fn a_rerun_leaves_an_installed_file_unchanged() {
    let dir = TempDir::new();
    install(&dir.agents_md()).expect("install should succeed");
    let first = fs::read_to_string(dir.agents_md()).unwrap();

    install(&dir.agents_md()).expect("a rerun should succeed");

    assert_eq!(fs::read_to_string(dir.agents_md()).unwrap(), first);
}

#[cfg(unix)]
#[test]
fn install_refuses_to_write_through_a_symlink() {
    let dir = TempDir::new();
    let target = dir.0.join("real.md");
    fs::write(&target, "# Elsewhere\n").unwrap();
    std::os::unix::fs::symlink(&target, dir.agents_md()).unwrap();

    let err = install(&dir.agents_md()).expect_err("a symlink target must be refused");
    assert!(
        err.to_string().contains("symlink"),
        "the error names the symlink; got: {err:#}"
    );
    assert_eq!(fs::read_to_string(&target).unwrap(), "# Elsewhere\n");
}

#[test]
fn an_unreadable_path_is_an_error_naming_the_read() {
    let dir = TempDir::new();
    fs::create_dir_all(dir.agents_md()).unwrap();

    let err = install(&dir.agents_md()).expect_err("a directory cannot be read as a file");
    assert!(
        format!("{err:#}").contains("reading"),
        "the error names the failed read; got: {err:#}"
    );
}

#[test]
fn install_refuses_a_begin_marker_with_no_end_marker() {
    let dir = TempDir::new();
    let damaged = "# My project\n\n\
         <!-- testing-conventions:begin v1 hash=000000000000 -->\n\
         ## Testing conventions\n\n\
         Important user prose that must survive.\n"
        .to_string();
    fs::write(dir.agents_md(), &damaged).unwrap();

    let result = install(&dir.agents_md());
    assert!(
        result.is_err(),
        "install must refuse a begin marker with no matching end marker"
    );

    let text = fs::read_to_string(dir.agents_md()).unwrap();
    assert_eq!(
        text, damaged,
        "the damaged file must be left untouched, not appended to"
    );
}
