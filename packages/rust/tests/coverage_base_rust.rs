use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::coverage::{Outcome, RustThresholds};
use testing_conventions::{patch_coverage, run};

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-cov-base-rust-{}-{}-{}",
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

fn floors(level: u8) -> RustThresholds {
    RustThresholds {
        regions: Some(level),
        lines: level,
        functions: None,
        branch: None,
    }
}

/// The diff-scoped outcome for `<base>...HEAD` at a uniform `level` floor (no
/// exemptions) via the SDK.
fn measure_base(repo: &TempRepo, base: &str, level: u8) -> Outcome {
    patch_coverage::measure_rust(
        &repo.0,
        base,
        floors(level),
        &[],
        &std::collections::BTreeMap::new(),
        &[],
    )
    .expect("measuring a readable repo should succeed")
}

/// Exit code of `unit coverage <repo> --language rust --base <base> [--config
/// <repo>/<config>]`, run in-process.
fn run_coverage_base(repo: &TempRepo, base: &str, config: Option<&str>) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "coverage".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        "rust".into(),
        "--base".into(),
        base.into(),
    ];
    if let Some(name) = config {
        argv.push("--config".into());
        argv.push(repo.0.join(name).into_os_string());
    }
    run(argv)
}

const CARGO_TOML: &str =
    "[package]\nname = \"tc_cov_base_rust\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[workspace]\n";

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
fn rust_a_diff_below_the_floor_fails() {
    let repo = TempRepo::new("below");
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    assert!(
        matches!(measure_base(&repo, &base, 80), Outcome::Fail(_)),
        "the diff's 50% regions / 50% lines are below an 80 floor"
    );
}

#[test]
fn rust_the_same_diff_clears_a_lower_floor() {
    let repo = TempRepo::new("clears");
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    assert_eq!(
        measure_base(&repo, &base, 40),
        Outcome::Pass,
        "both metrics (50%) clear a 40 floor despite the uncovered arm"
    );
}

#[test]
fn rust_a_fully_covered_change_passes() {
    let repo = TempRepo::new("covered");
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_COVERED_EDIT);
    repo.commit("reword a covered line and update its test");

    assert_eq!(measure_base(&repo, &base, 80), Outcome::Pass);
}

#[test]
fn rust_a_tiny_below_floor_diff_is_not_exempted() {
    let repo = TempRepo::new("tiny");
    let base = baseline(&repo);
    repo.write("src/lib.rs", &format!("{WIDGET_RS}pub mod lonely;\n"));
    repo.write("src/lonely.rs", "pub fn lonely() -> i64 {\n    41\n}\n");
    repo.commit("add one untested module");

    assert!(
        matches!(measure_base(&repo, &base, 80), Outcome::Fail(_)),
        "a tiny 0%-covered diff still fails an 80 floor"
    );
}

#[test]
fn rust_a_change_touching_no_rust_passes() {
    let repo = TempRepo::new("no-rs");
    repo.write("src/lib.rs", WIDGET_RS);
    repo.write("README.md", "# project\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("README.md", "# project\n\nnow with docs\n");
    repo.commit("docs only");

    assert_eq!(measure_base(&repo, &base, 100), Outcome::Pass);
}

#[test]
fn rust_an_unknown_base_ref_is_an_error() {
    let repo = TempRepo::new("bad-base");
    let _ = baseline(&repo);
    assert!(
        patch_coverage::measure_rust(
            &repo.0,
            "no-such-ref",
            floors(80),
            &[],
            &std::collections::BTreeMap::new(),
            &[]
        )
        .is_err(),
        "an unresolvable base ref must error"
    );
}

#[test]
fn rust_cli_exits_nonzero_on_a_below_floor_diff() {
    let repo = TempRepo::new("cli-red");
    repo.write("testing-conventions.toml", &config_toml(80));
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        1
    );
}

#[test]
fn rust_cli_exits_zero_when_the_diff_clears_the_floor() {
    let repo = TempRepo::new("cli-clean");
    repo.write("testing-conventions.toml", &config_toml(80));
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_COVERED_EDIT);
    repo.commit("reword a covered line and update its test");

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn rust_cli_a_lower_configured_floor_lets_the_same_diff_pass() {
    let repo = TempRepo::new("cli-floor40");
    repo.write("testing-conventions.toml", &config_toml(40));
    let base = baseline(&repo);
    repo.write("src/lib.rs", WIDGET_RS_UNCOVERED);
    repo.commit("add an untested arm");

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn rust_cli_a_docs_only_diff_passes() {
    let repo = TempRepo::new("cli-docs");
    repo.write("testing-conventions.toml", &config_toml(80));
    repo.write("src/lib.rs", WIDGET_RS);
    repo.write("README.md", "# project\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("README.md", "# project\n\nnow with docs\n");
    repo.commit("docs only");

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn rust_a_coverage_exemption_lifts_a_below_floor_change() {
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[rust.coverage]\nregions = 80\nlines = 80\n\n\
         [[rust.exempt]]\npath = \"src/shim.rs\"\nrules = [\"coverage\"]\n\
         lines = [\"1-3\"]\nreason = \"thin launcher; logic lives in tested modules\"\n",
    );
    repo.write("src/lib.rs", &format!("{WIDGET_RS}pub mod shim;\n"));
    repo.write("src/shim.rs", "pub fn shim() -> i64 {\n    0\n}\n");
    repo.commit("base");
    let base = repo.head();

    repo.write("src/shim.rs", "pub fn shim() -> i64 {\n    1\n}\n");
    repo.commit("edit the untested launcher");

    let floor_only = TempRepo::new("exempt-floor-only");
    floor_only.write("testing-conventions.toml", &config_toml(80));
    floor_only.write("src/lib.rs", &format!("{WIDGET_RS}pub mod shim;\n"));
    floor_only.write("src/shim.rs", "pub fn shim() -> i64 {\n    0\n}\n");
    floor_only.commit("base");
    let floor_only_base = floor_only.head();
    floor_only.write("src/shim.rs", "pub fn shim() -> i64 {\n    1\n}\n");
    floor_only.commit("edit the untested launcher");
    assert_eq!(
        run_coverage_base(
            &floor_only,
            &floor_only_base,
            Some("testing-conventions.toml")
        )
        .unwrap(),
        1,
        "the untested shim edit is below the 80 floor without the exemption"
    );

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0,
        "the `coverage` exemption drops the shim, lifting its changed lines"
    );
}
