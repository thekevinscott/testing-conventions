//! E2E tests for the exemption-approval gate's detection (#229): drive the built
//! `testing-conventions exemptions --base` binary as a real subprocess against
//! throwaway git repos and assert the exit code (and, for the red case, the named
//! offender plus the `tc:exemption-approved` greenlight hint). Complements the
//! in-process integration tests in `exemptions.rs`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway git repo, removed on drop. A test writes the "before" config,
/// `commit`s it, captures `head()` as the `base`, then writes the "after" and commits.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-exemptions-e2e-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        TempRepo(root)
    }

    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    fn head(&self) -> String {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        assert!(out.status.success(), "git rev-parse failed");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git should run");
    assert!(status.success(), "git {args:?} failed");
}

/// Exit code + combined stdout/stderr of `exemptions --base <base> [--config …]
/// [--approved]`, run as a real subprocess against the built binary, with cwd set to the
/// repo. (Combined because the approved audit trail is printed to stdout, the red error
/// to stderr.)
fn exemptions(
    repo: &TempRepo,
    base: &str,
    with_config_flag: bool,
    approved: bool,
) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.current_dir(&repo.0)
        .arg("exemptions")
        .args(["--base", base]);
    if with_config_flag {
        cmd.args(["--config", "testing-conventions.toml"]);
    }
    if approved {
        cmd.arg("--approved");
    }
    let output = cmd.output().expect("the built binary should run");
    let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
    combined.push_str(&String::from_utf8_lossy(&output.stderr));
    (
        output
            .status
            .code()
            .expect("the process should exit with a code"),
        combined,
    )
}

const CONFIG: &str = "testing-conventions.toml";
const COVERAGE: &str = "[python]\ncoverage = { branch = true, fail_under = 100 }\n";

const WITH_EXEMPT: &str = "[python]\ncoverage = { branch = true, fail_under = 100 }\n\n\
     [[python.exempt]]\npath = \"mypkg/cli.py\"\nrules = [\"colocated-test\"]\nreason = \"thin launcher\"\n";

// A line-scoped `coverage` exemption (#226) over lines 12-13.
const WITH_LINES_12_13: &str = "[python]\ncoverage = { branch = true, fail_under = 100 }\n\n\
     [[python.exempt]]\npath = \"mypkg/cfg.py\"\nrules = [\"coverage\"]\nlines = [\"12-13\"]\nreason = \"dead branch\"\n";

// The same entry widened to lines 12-200 — a modification that must gate.
const WITH_LINES_12_200: &str = "[python]\ncoverage = { branch = true, fail_under = 100 }\n\n\
     [[python.exempt]]\npath = \"mypkg/cfg.py\"\nrules = [\"coverage\"]\nlines = [\"12-200\"]\nreason = \"dead branch\"\n";

#[test]
fn a_new_exemption_exits_nonzero_and_names_it_with_the_label_hint() {
    let repo = TempRepo::new("red");
    repo.write(CONFIG, COVERAGE);
    repo.commit("base: no exemptions");
    let base = repo.head();
    repo.write(CONFIG, WITH_EXEMPT);
    repo.commit("add an exemption");

    let (code, stderr) = exemptions(&repo, &base, true, false);
    assert_eq!(
        code, 1,
        "a newly-added exemption must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("mypkg/cli.py"),
        "stderr should name the new exemption; got: {stderr}"
    );
    assert!(
        stderr.contains("tc:exemption-approved"),
        "stderr should point at the greenlight label; got: {stderr}"
    );
}

#[test]
fn an_unchanged_config_exits_zero() {
    let repo = TempRepo::new("clean");
    repo.write(CONFIG, WITH_EXEMPT);
    repo.commit("base: one exemption");
    let base = repo.head();
    repo.write("README.md", "# hi\n");
    repo.commit("unrelated change");

    let (code, stderr) = exemptions(&repo, &base, true, false);
    assert_eq!(code, 0, "no new exemption must exit zero; stderr: {stderr}");
}

#[test]
fn removing_an_exemption_exits_zero() {
    let repo = TempRepo::new("remove");
    repo.write(CONFIG, WITH_EXEMPT);
    repo.commit("base: one exemption");
    let base = repo.head();
    repo.write(CONFIG, COVERAGE);
    repo.commit("remove it");

    let (code, stderr) = exemptions(&repo, &base, true, false);
    assert_eq!(
        code, 0,
        "removing an exemption must exit zero; stderr: {stderr}"
    );
}

#[test]
fn the_config_flag_defaults_to_testing_conventions_toml() {
    // No `--config`: the default path (`testing-conventions.toml`, resolved against
    // cwd) is read, so the drop-in needs no flag.
    let repo = TempRepo::new("default-config");
    repo.write(CONFIG, COVERAGE);
    repo.commit("base");
    let base = repo.head();
    repo.write(CONFIG, WITH_EXEMPT);
    repo.commit("add an exemption");

    let (code, stderr) = exemptions(&repo, &base, false, false);
    assert_eq!(
        code, 1,
        "the default config path must be read; stderr: {stderr}"
    );
}

#[test]
fn the_approved_greenlight_lets_a_new_exemption_pass_with_an_audit_line() {
    // The binary half of the human greenlight: with --approved (which the workflow sets
    // only when a non-author reviewer applied the label), the same diff that was red
    // passes — and the approved exemption is echoed as an audit trail.
    let repo = TempRepo::new("approved");
    repo.write(CONFIG, COVERAGE);
    repo.commit("base: no exemptions");
    let base = repo.head();
    repo.write(CONFIG, WITH_EXEMPT);
    repo.commit("add an exemption");

    // Red without the greenlight…
    assert_eq!(exemptions(&repo, &base, true, false).0, 1);
    // …green with it, and the approved exemption is listed.
    let (code, out) = exemptions(&repo, &base, true, true);
    assert_eq!(code, 0, "an approved exemption must pass; output: {out}");
    assert!(
        out.contains("mypkg/cli.py") && out.contains("approved"),
        "an audit line should name the approved exemption; got: {out}"
    );
}

#[test]
fn widening_a_line_scope_exits_nonzero_and_names_the_lines() {
    // The #226 anti-broadening case, end to end: widening a line-scoped coverage exemption
    // gates, and the offending entry (with its widened lines) is named.
    let repo = TempRepo::new("widen");
    repo.write(CONFIG, WITH_LINES_12_13);
    repo.commit("base: exempt lines 12-13");
    let base = repo.head();
    repo.write(CONFIG, WITH_LINES_12_200);
    repo.commit("widen to 12-200");

    let (code, out) = exemptions(&repo, &base, true, false);
    assert_eq!(code, 1, "widening a line scope must gate; output: {out}");
    assert!(
        out.contains("mypkg/cfg.py") && out.contains("lines"),
        "the widened entry and its lines should be named; got: {out}"
    );
}

#[test]
fn an_unresolvable_base_ref_fails() {
    let repo = TempRepo::new("bad-base");
    repo.write(CONFIG, COVERAGE);
    repo.commit("base");

    let (code, _stderr) = exemptions(&repo, "no-such-ref", true, false);
    assert_ne!(code, 0, "an unresolvable base ref must not pass as clean");
}
