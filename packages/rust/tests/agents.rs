//! Integration tests for the `agents` upsert engine (#232): drive the CLI
//! in-process (`testing_conventions::run`) against temp files and assert exit
//! codes and file effects. The stdout words (`installed` / `current` / …) are
//! asserted by the subprocess e2e suite in `agents_e2e.rs`.

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

const BEGIN_PREFIX: &str = "<!-- testing-conventions:begin v1 hash=";
const END_MARKER: &str = "<!-- testing-conventions:end -->";

/// A fresh scratch dir, unique per test, so parallel tests never collide.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tc-agents-{}-{}", name, std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("stale scratch dir should be removable");
    }
    fs::create_dir_all(&dir).expect("scratch dir should be creatable");
    dir
}

/// Exit code of `agents <verb> <path>`, run in-process.
fn agents(verb: &str, path: &Path) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "agents".into(),
        verb.into(),
        path.into(),
    ];
    testing_conventions::run(argv).expect("the agents command should be recognized")
}

/// The managed region (markers included), extracted from the file.
fn managed_region(contents: &str) -> &str {
    let begin = contents
        .find(BEGIN_PREFIX)
        .expect("the file should carry the begin marker");
    let end = contents
        .find(END_MARKER)
        .expect("the file should carry the end marker");
    &contents[begin..end + END_MARKER.len()]
}

#[test]
fn install_creates_a_missing_file() {
    let dir = scratch("create");
    let path = dir.join("AGENTS.md");

    assert_eq!(agents("install", &path), 0);

    let contents = fs::read_to_string(&path).expect("install should create the file");
    let begin_line = contents
        .lines()
        .find(|l| l.starts_with(BEGIN_PREFIX))
        .expect("the begin marker should be a full line");
    // The begin marker carries the schema version and a 12-hex content hash.
    let hash = begin_line
        .strip_prefix(BEGIN_PREFIX)
        .and_then(|rest| rest.strip_suffix(" -->"))
        .expect("the begin marker should close with ' -->'");
    assert_eq!(hash.len(), 12, "hash should be 12 chars: {begin_line:?}");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "hash should be hex: {begin_line:?}"
    );
    assert!(contents.contains(END_MARKER));
}

#[test]
fn install_appends_when_markers_are_absent() {
    let dir = scratch("append");
    let path = dir.join("AGENTS.md");
    let existing = "# My project\n\nHand-written agent notes.\n";
    fs::write(&path, existing).expect("seeding the file should work");

    assert_eq!(agents("install", &path), 0);

    let contents = fs::read_to_string(&path).expect("the file should still exist");
    assert!(
        contents.starts_with(existing),
        "existing content should be preserved byte-for-byte at the top"
    );
    assert!(contents.contains(BEGIN_PREFIX));
    assert!(contents.contains(END_MARKER));
}

#[test]
fn install_replaces_only_the_managed_region() {
    let dir = scratch("replace");
    let path = dir.join("AGENTS.md");
    let before = "# Intro kept as-is\n\n";
    let after = "\n## Outro kept as-is\n";
    let stale_block = format!(
        "{BEGIN_PREFIX}000000000000 -->\nOld contract text from a previous release.\n{END_MARKER}"
    );
    fs::write(&path, format!("{before}{stale_block}{after}"))
        .expect("seeding the file should work");

    assert_eq!(agents("install", &path), 0);

    let contents = fs::read_to_string(&path).expect("the file should still exist");
    assert!(
        contents.starts_with(before),
        "content before the region should be untouched"
    );
    assert!(
        contents.ends_with(after),
        "content after the region should be untouched"
    );
    assert!(
        !contents.contains("Old contract text"),
        "the stale region body should be gone"
    );
    assert!(
        !contents.contains("hash=000000000000"),
        "the stale hash should be rewritten"
    );
}

#[test]
fn reinstall_is_a_byte_identical_no_op() {
    let dir = scratch("noop");
    let path = dir.join("AGENTS.md");

    assert_eq!(agents("install", &path), 0);
    let first = fs::read(&path).expect("install should create the file");

    assert_eq!(agents("install", &path), 0);
    let second = fs::read(&path).expect("the file should still exist");
    assert_eq!(first, second, "a current block should not be rewritten");
}

#[test]
fn check_passes_on_a_current_block() {
    let dir = scratch("check-current");
    let path = dir.join("AGENTS.md");

    assert_eq!(agents("install", &path), 0);
    assert_eq!(agents("check", &path), 0);
}

#[test]
fn check_flags_a_tampered_region_as_stale() {
    let dir = scratch("check-stale");
    let path = dir.join("AGENTS.md");

    assert_eq!(agents("install", &path), 0);
    let contents = fs::read_to_string(&path).expect("install should create the file");
    let region = managed_region(&contents).to_string();
    let tampered_region = region.replace(END_MARKER, &format!("Hand edit.\n{END_MARKER}"));
    fs::write(&path, contents.replace(&region, &tampered_region)).expect("tampering should write");

    assert_eq!(agents("check", &path), 1, "a hand-edited region is stale");

    // install repairs it, after which check is clean again.
    assert_eq!(agents("install", &path), 0);
    assert_eq!(agents("check", &path), 0);
}

#[test]
fn check_flags_a_missing_file_as_absent() {
    let dir = scratch("check-no-file");
    assert_eq!(agents("check", &dir.join("AGENTS.md")), 1);
}

#[test]
fn check_flags_a_file_without_markers_as_absent() {
    let dir = scratch("check-no-markers");
    let path = dir.join("AGENTS.md");
    fs::write(&path, "# Notes, no managed block.\n").expect("seeding the file should work");
    assert_eq!(agents("check", &path), 1);
}

#[test]
fn remove_deletes_only_the_managed_region() {
    let dir = scratch("remove");
    let path = dir.join("AGENTS.md");
    let existing = "# My project\n\nHand-written agent notes.\n";
    fs::write(&path, existing).expect("seeding the file should work");
    assert_eq!(agents("install", &path), 0);

    assert_eq!(agents("remove", &path), 0);

    let contents = fs::read_to_string(&path).expect("the file should still exist");
    assert!(
        !contents.contains(BEGIN_PREFIX) && !contents.contains(END_MARKER),
        "the markers should be gone"
    );
    assert!(
        contents.contains("Hand-written agent notes."),
        "hand-written content should survive removal"
    );

    // Removal is idempotent: a second remove still exits 0.
    assert_eq!(agents("remove", &path), 0);
}

#[cfg(unix)]
#[test]
fn install_refuses_a_symlinked_target() {
    let dir = scratch("symlink-install");
    let real = dir.join("real.md");
    fs::write(&real, "# Real file\n").expect("seeding the target should work");
    let link = dir.join("AGENTS.md");
    std::os::unix::fs::symlink(&real, &link).expect("symlinking should work");

    assert_eq!(agents("install", &link), 1);

    let contents = fs::read_to_string(&real).expect("the target should still exist");
    assert_eq!(contents, "# Real file\n", "the symlink target is untouched");
    assert!(
        link.symlink_metadata()
            .expect("the link should still exist")
            .file_type()
            .is_symlink(),
        "the symlink itself is untouched"
    );
}

#[cfg(unix)]
#[test]
fn remove_refuses_a_symlinked_target() {
    let dir = scratch("symlink-remove");
    let real = dir.join("real.md");
    fs::write(&real, "# Real file\n").expect("seeding the target should work");
    let link = dir.join("AGENTS.md");
    std::os::unix::fs::symlink(&real, &link).expect("symlinking should work");

    assert_eq!(agents("remove", &link), 1);
    let contents = fs::read_to_string(&real).expect("the target should still exist");
    assert_eq!(contents, "# Real file\n", "the symlink target is untouched");
}
