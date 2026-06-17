//! E2E tests for diff-scoped Rust coverage — `unit coverage --language rust
//! --base` (#162): drive the built CLI binary as a real subprocess against
//! throwaway cargo crates (each a git repo) and assert the exit code (and, for a red
//! case, the failure on stderr). Complements the in-process integration tests in
//! `coverage_base_rust.rs`. Each crate carries its own `[workspace]` so `cargo
//! llvm-cov` measures it in isolation; Rust has no zero-config default floor, so
//! every case commits a `[rust.coverage]` table. Requires `git` + `cargo-llvm-cov`
//! on PATH (the runs are slow — building and instrumenting each crate from scratch).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway cargo crate in a git repo, removed on drop. A test writes a baseline,
/// `commit`s it, captures `head()` as the `base`, then mutates and commits the
/// "after". The crate carries its own `[workspace]` so `cargo llvm-cov` measures it
/// in isolation.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-cov-base-rust-e2e-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        let repo = TempRepo(root);
        repo.write("Cargo.toml", CARGO_TOML);
        repo
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

/// Exit code + stderr of `unit coverage <repo> --language rust --base <base>
/// [--config <repo>/<config>]`, run as a real subprocess.
fn coverage_base(repo: &TempRepo, base: &str, config: Option<&str>) -> (i32, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_testing-conventions"));
    cmd.arg("unit")
        .arg("coverage")
        .arg(&repo.0)
        .args(["--language", "rust", "--base", base]);
    if let Some(name) = config {
        cmd.arg("--config").arg(repo.0.join(name));
    }
    let output = cmd.output().expect("the built binary should run");
    (
        output
            .status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

const CARGO_TOML: &str =
    "[package]\nname = \"tc_cov_base_rust_e2e\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n";

/// A `[rust.coverage]` config at the given uniform floor — committed so the
/// measurement is deterministic (Rust has no zero-config default).
fn config_toml(level: u8) -> String {
    format!("[rust.coverage]\nregions = {level}\nlines = {level}\n")
}

const WIDGET_RS: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "pos"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "pos");
        assert_eq!(widget(-1), "neg");
    }
}
"#;

/// After: an `else if n == -42` arm the suite never exercises → the diff (new lines
/// 4-5) lands at regions 50% / lines 50% (see `coverage_base_rust.rs`), so it is
/// below an 80 floor and clears a 40 floor.
const WIDGET_RS_UNCOVERED: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "pos"
    } else if n == -42 {
        "answer"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "pos");
        assert_eq!(widget(-1), "neg");
    }
}
"#;

const WIDGET_RS_COVERED_EDIT: &str = r#"pub fn widget(n: i64) -> &'static str {
    if n > 0 {
        "positive"
    } else {
        "neg"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_arms() {
        assert_eq!(widget(1), "positive");
        assert_eq!(widget(-1), "neg");
    }
}
"#;

fn baseline(repo: &TempRepo) -> String {
    repo.write("src/lib.rs", WIDGET_RS);
    repo.commit("base");
    repo.head()
}

#[test]
fn rust_below_floor_diff_exits_nonzero_and_reports_coverage() {
    let repo = TempRepo::new("red");
    repo.write("testing-conventions.toml", &config_toml(80));
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    let (code, stderr) = coverage_base(&repo, &base, Some("testing-conventions.toml"));
    assert_eq!(
        code, 1,
        "a diff below the floor must exit non-zero; stderr: {stderr}"
    );
    assert!(
        stderr.contains("coverage"),
        "stderr should report the coverage shortfall; got: {stderr}"
    );
}

#[test]
fn rust_covered_change_exits_zero() {
    let repo = TempRepo::new("clean");
    repo.write("testing-conventions.toml", &config_toml(80));
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_COVERED_EDIT);
    repo.commit("reword a covered line and update its test");

    let (code, stderr) = coverage_base(&repo, &base, Some("testing-conventions.toml"));
    assert_eq!(code, 0, "a fully covered change passes; stderr: {stderr}");
}

#[test]
fn rust_a_lower_configured_floor_lets_the_same_diff_pass() {
    // The behavior change: the diff that fails an 80 floor passes once the configured
    // floors are 40 — the floor is the single source of truth. The config is committed
    // so the measurement is deterministic.
    let repo = TempRepo::new("floor40");
    repo.write("testing-conventions.toml", &config_toml(40));
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    let (code, stderr) = coverage_base(&repo, &base, Some("testing-conventions.toml"));
    assert_eq!(
        code, 0,
        "the diff (50%) clears a configured 40 floor; stderr: {stderr}"
    );
}

#[test]
fn rust_a_tiny_below_floor_diff_still_exits_nonzero() {
    // No small-diff carve-out (#162): a single untested module (the suite never
    // exercises it → 0% on its lines) fails an 80 floor.
    let repo = TempRepo::new("tiny");
    repo.write("testing-conventions.toml", &config_toml(80));
    let base = baseline(&repo);
    repo.write("src/lib.rs", &format!("{WIDGET_RS}pub mod lonely;\n"));
    repo.write("src/lonely.rs", "pub fn lonely() -> i64 {\n    41\n}\n");
    repo.commit("add one untested module");

    let (code, stderr) = coverage_base(&repo, &base, Some("testing-conventions.toml"));
    assert_eq!(
        code, 1,
        "a tiny diff below the floor is not exempted; stderr: {stderr}"
    );
}
