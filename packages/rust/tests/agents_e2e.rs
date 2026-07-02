//! E2E tests for the `agents` command group (#232): drive the built CLI binary
//! as a real subprocess against temp dirs with assorted starting `AGENTS.md`
//! states, asserting the stdout word and the exit code. Complements the
//! in-process integration tests in `agents.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh scratch dir, unique per test, so parallel tests never collide.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("tc-agents-e2e-{}-{}", name, std::process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("stale scratch dir should be removable");
    }
    fs::create_dir_all(&dir).expect("scratch dir should be creatable");
    dir
}

/// (exit code, stdout) of `agents <verb> [path]` run from `cwd` as a subprocess.
fn agents_in(cwd: &Path, verb: &str, path: Option<&Path>) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.current_dir(cwd).args(["agents", verb]);
    if let Some(path) = path {
        cmd.arg(path);
    }
    let output = cmd.output().expect("the built binary should run");
    (
        output
            .status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8(output.stdout).expect("stdout should be UTF-8"),
    )
}

/// The single status word the command prints on stdout.
fn word(stdout: &str) -> String {
    stdout.trim().to_string()
}

#[test]
fn fresh_install_prints_installed_and_rerun_prints_current() {
    let dir = scratch("install-words");
    let path = dir.join("AGENTS.md");

    let (code, out) = agents_in(&dir, "install", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "installed"));

    let (code, out) = agents_in(&dir, "install", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "current"));
}

#[test]
fn the_path_defaults_to_agents_md_in_the_cwd() {
    let dir = scratch("default-path");

    let (code, _) = agents_in(&dir, "install", None);
    assert_eq!(code, 0);
    assert!(
        dir.join("AGENTS.md").is_file(),
        "install with no path should manage ./AGENTS.md"
    );

    let (code, out) = agents_in(&dir, "check", None);
    assert_eq!((code, word(&out).as_str()), (0, "current"));
}

#[test]
fn check_reports_absent_stale_and_current_across_the_lifecycle() {
    let dir = scratch("lifecycle");
    let path = dir.join("AGENTS.md");

    // No file yet.
    let (code, out) = agents_in(&dir, "check", Some(&path));
    assert_eq!((code, word(&out).as_str()), (1, "absent"));

    // A file without markers is just as absent.
    fs::write(&path, "# Notes, no managed block.\n").expect("seeding the file should work");
    let (code, out) = agents_in(&dir, "check", Some(&path));
    assert_eq!((code, word(&out).as_str()), (1, "absent"));

    // Installed → current.
    let (code, out) = agents_in(&dir, "install", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "installed"));
    let (code, out) = agents_in(&dir, "check", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "current"));

    // A block from an older template (markers present, different content) is stale…
    let contents = fs::read_to_string(&path).expect("the file should exist");
    let tampered = contents.replace(
        "<!-- testing-conventions:end -->",
        "An extra hand-written line.\n<!-- testing-conventions:end -->",
    );
    assert_ne!(contents, tampered, "the region edit should take");
    fs::write(&path, tampered).expect("tampering should write");
    let (code, out) = agents_in(&dir, "check", Some(&path));
    assert_eq!((code, word(&out).as_str()), (1, "stale"));

    // …and reinstalling repairs it.
    let (code, out) = agents_in(&dir, "install", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "updated"));
    let (code, out) = agents_in(&dir, "check", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "current"));
}

#[test]
fn remove_prints_removed_then_absent() {
    let dir = scratch("remove-words");
    let path = dir.join("AGENTS.md");
    fs::write(&path, "# Kept prose.\n").expect("seeding the file should work");

    let (code, _) = agents_in(&dir, "install", Some(&path));
    assert_eq!(code, 0);

    let (code, out) = agents_in(&dir, "remove", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "removed"));
    assert!(
        fs::read_to_string(&path)
            .expect("the file should still exist")
            .contains("Kept prose."),
        "hand-written content should survive removal"
    );

    let (code, out) = agents_in(&dir, "remove", Some(&path));
    assert_eq!((code, word(&out).as_str()), (0, "absent"));
}

#[cfg(unix)]
#[test]
fn a_symlinked_target_is_refused_with_a_warning() {
    let dir = scratch("symlink");
    let real = dir.join("real.md");
    fs::write(&real, "# Real file\n").expect("seeding the target should work");
    let link = dir.join("AGENTS.md");
    std::os::unix::fs::symlink(&real, &link).expect("symlinking should work");

    let output = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .current_dir(&dir)
        .args(["agents", "install"])
        .arg(&link)
        .output()
        .expect("the built binary should run");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("symlink"),
        "the refusal should name the reason: {stderr:?}"
    );
    assert_eq!(
        fs::read_to_string(&real).expect("the target should still exist"),
        "# Real file\n",
        "the symlink target is untouched"
    );
}
