//! Integration tests for diff-scoped coverage — `unit coverage --base`.
//!
//! Folds the old `unit patch-coverage` into the coverage floor: with `--base`, the
//! SAME configured floor is measured over the `<base>...HEAD` diff (the changed
//! lines) instead of the whole tree. Unlike the implicit-100% patch-coverage it
//! replaces, a changed line is judged against the configured floor — a diff that
//! clears it passes even with an uncovered line, and one below it fails however
//! small the diff (no small-diff carve-out).
//!
//! Each test builds a throwaway git repo (the codebases are the fixtures) and
//! runs REAL coverage.py over it via the SDK
//! (`patch_coverage::measure`) and the CLI (`run`). Requires `coverage` + `pytest`
//! + `git` on PATH.
//!
//! The default repo is the prescribed consumer package layout — `{pyproject.toml,
//! src/**}` scanned at `<repo>/src`. The `--base` diff is still computed over the
//! whole repo; the scan path handed to the SDK/CLI is `src/`.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::coverage::{Outcome, Thresholds};
use testing_conventions::{patch_coverage, run};

/// A throwaway git repo, removed on drop. A test writes a baseline source + its
/// colocated test, `commit`s it, captures `head()` as the `base`, then mutates and
/// commits the "after" so `<base>...HEAD` is the change under test.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-cov-base-{}-{}-{}",
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

    /// Write `contents` to `rel`, creating parent directories.
    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// Stage everything and commit, advancing HEAD.
    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    /// The current HEAD SHA — captured as the `base` before mutating.
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

/// The diff-scoped outcome for `<base>...HEAD` at `fail_under` (branch on, no
/// exemptions) via the SDK.
fn measure_base(repo: &TempRepo, base: &str, fail_under: u8) -> Outcome {
    patch_coverage::measure(
        &repo.0.join("src"),
        base,
        Thresholds {
            fail_under,
            branch: true,
        },
        &[],
        &std::collections::BTreeMap::new(),
    )
    .expect("measuring a readable repo should succeed")
}

/// Exit code of `unit coverage <repo> --language python --base <base> [--config
/// <repo>/<config>]`, run in-process.
fn run_coverage_base(repo: &TempRepo, base: &str, config: Option<&str>) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "coverage".into(),
        repo.0.join("src").into_os_string(),
        "--language".into(),
        "python".into(),
        "--base".into(),
        base.into(),
    ];
    if let Some(name) = config {
        argv.push("--config".into());
        argv.push(repo.0.join(name).into_os_string());
    }
    run(argv)
}

/// The package root: anchors pytest's rootdir at `<repo>` so the colocated suite
/// under `src/` resolves its `from widget import ...` when coverage runs at the scan
/// path `<repo>/src`.
const PYPROJECT: &str = "[tool.pytest.ini_options]\n";

/// Baseline: `widget` is fully covered (both branches taken) by `WIDGET_TEST_PY`.
const WIDGET_PY: &str = r#"def widget(n):
    if n > 0:
        return "pos"
    return "neg"
"#;
const WIDGET_TEST_PY: &str = r#"from widget import widget


def test_widget():
    assert widget(1) == "pos"
    assert widget(-1) == "neg"
"#;

/// After: appends two one-line functions — `covered` (the test calls it) and
/// `uncovered` (it doesn't). Four new *executable* lines: each `def` runs at
/// import, `covered`'s `return` runs, `uncovered`'s never does → 3 / 4 = **75%**
/// covered on the diff (the appended blanks aren't executable). So the same diff
/// clears a 70 floor but fails an 85 floor.
const WIDGET_PY_75: &str = r#"def widget(n):
    if n > 0:
        return "pos"
    return "neg"


def covered():
    return 1


def uncovered():
    return 2
"#;
const WIDGET_TEST_75: &str = r#"from widget import widget, covered


def test_widget():
    assert widget(1) == "pos"
    assert widget(-1) == "neg"


def test_covered():
    assert covered() == 1
"#;

/// Writes the fully-covered baseline in the default package layout —
/// `{pyproject.toml, src/**}` — and returns its commit as the base ref.
fn baseline(repo: &TempRepo) -> String {
    repo.write("pyproject.toml", PYPROJECT);
    repo.write("src/widget.py", WIDGET_PY);
    repo.write("src/widget_test.py", WIDGET_TEST_PY);
    repo.commit("base");
    repo.head()
}

#[test]
fn a_diff_below_the_floor_fails() {
    // The core red case: the 75%-covered diff is below the 85 floor under test,
    // so `--base` fails it — even though the whole tree is still well covered.
    let repo = TempRepo::new("below");
    let base = baseline(&repo);
    repo.write("src/widget.py", WIDGET_PY_75);
    repo.write("src/widget_test.py", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert!(
        matches!(measure_base(&repo, &base, 85), Outcome::Fail(_)),
        "75% on the diff is below an 85 floor"
    );
}

#[test]
fn the_same_diff_clears_a_lower_floor() {
    // The behavior change from the implicit-100% patch-coverage: the SAME diff,
    // with its one uncovered line, PASSES once the configured floor is 70 — the
    // changed lines are judged against the number you set, not against 100%.
    let repo = TempRepo::new("clears");
    let base = baseline(&repo);
    repo.write("src/widget.py", WIDGET_PY_75);
    repo.write("src/widget_test.py", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(
        measure_base(&repo, &base, 70),
        Outcome::Pass,
        "75% on the diff clears a 70 floor despite the uncovered line"
    );
}

#[test]
fn a_fully_covered_change_passes() {
    // Editing a line the suite already exercises keeps the diff at 100% → any
    // floor is met.
    let repo = TempRepo::new("covered");
    let base = baseline(&repo);
    repo.write(
        "src/widget.py",
        r#"def widget(n):
    if n > 0:
        return "positive"
    return "neg"
"#,
    );
    repo.write(
        "src/widget_test.py",
        r#"from widget import widget


def test_widget():
    assert widget(1) == "positive"
    assert widget(-1) == "neg"
"#,
    );
    repo.commit("reword a covered line and update its test");

    assert_eq!(measure_base(&repo, &base, 85), Outcome::Pass);
}

#[test]
fn a_tiny_below_floor_diff_is_not_exempted() {
    // There is no small-diff carve-out. A two-line diff (a
    // single untested helper: the `def` runs at import, its `return` never does →
    // 50%) fails the 85 floor just like a large one would.
    let repo = TempRepo::new("tiny");
    let base = baseline(&repo);
    repo.write(
        "src/widget.py",
        &format!("{WIDGET_PY}\n\ndef lonely():\n    return 41\n"),
    );
    repo.commit("add one untested helper");

    assert!(
        matches!(measure_base(&repo, &base, 85), Outcome::Fail(_)),
        "a tiny 50%-covered diff still fails an 85 floor"
    );
}

#[test]
fn a_change_touching_no_python_passes() {
    // A diff with no `.py` source has no changed line to measure — vacuously
    // passes (the suite isn't even run), at any floor.
    let repo = TempRepo::new("no-py");
    repo.write("pyproject.toml", PYPROJECT);
    repo.write("src/widget.py", WIDGET_PY);
    repo.write("src/widget_test.py", WIDGET_TEST_PY);
    repo.write("README.md", "# project\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("README.md", "# project\n\nnow with docs\n");
    repo.commit("docs only");

    assert_eq!(measure_base(&repo, &base, 100), Outcome::Pass);
}

#[test]
fn an_unknown_base_ref_is_an_error() {
    // A base that can't be resolved must surface, never silently pass as "clean".
    let repo = TempRepo::new("bad-base");
    let _ = baseline(&repo);
    assert!(
        patch_coverage::measure(
            &repo.0.join("src"),
            "no-such-ref",
            Thresholds {
                fail_under: 85,
                branch: true,
            },
            &[],
            &std::collections::BTreeMap::new(),
        )
        .is_err(),
        "an unresolvable base ref must error"
    );
}

#[test]
fn a_plus_plus_line_keeps_the_uncovered_change_in_scope() {
    // #392: a `++ 1` line renders as `+++ 1` in the diff and was consumed by the
    // `+++` file-header branch, diverting the file's later added lines (an untested
    // `never_run`) to a bogus key — dropping them from the ratio, so a below-floor
    // change passed vacuously. The uncovered `return 999` must stay in scope, failing
    // the 100 floor.
    let repo = TempRepo::new("plusplus");
    repo.write("pyproject.toml", PYPROJECT);
    repo.write("src/calc.py", "def calc(n):\n    return n\n");
    repo.write(
        "src/calc_test.py",
        "from calc import calc\n\n\ndef test_calc():\n    assert calc(3) == 3\n",
    );
    repo.commit("base");
    let base = repo.head();
    repo.write(
        "src/calc.py",
        "def calc(n):\n    return n\n\n\n++ 1\n\n\ndef never_run():\n    return 999\n",
    );
    repo.commit("append a ++ line and an untested helper");

    assert!(
        matches!(measure_base(&repo, &base, 100), Outcome::Fail(_)),
        "the uncovered line after the ++ line must stay in scope and fail the floor"
    );
}

#[test]
fn cli_exits_nonzero_on_a_below_floor_diff() {
    // No config, so the diff is judged against the default Python floor (now 100);
    // the 75% diff is below it → exit 1.
    let repo = TempRepo::new("cli-red");
    let base = baseline(&repo);
    repo.write("src/widget.py", WIDGET_PY_75);
    repo.write("src/widget_test.py", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 1);
}

#[test]
fn cli_exits_zero_when_the_diff_clears_the_floor() {
    let repo = TempRepo::new("cli-clean");
    let base = baseline(&repo);
    repo.write(
        "src/widget.py",
        r#"def widget(n):
    if n > 0:
        return "positive"
    return "neg"
"#,
    );
    repo.write(
        "src/widget_test.py",
        r#"from widget import widget


def test_widget():
    assert widget(1) == "positive"
    assert widget(-1) == "neg"
"#,
    );
    repo.commit("reword a covered line and update its test");

    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 0);
}

#[test]
fn cli_a_lower_configured_floor_lets_the_same_diff_pass() {
    // A `[python.coverage] fail_under = 70` config re-scopes the floor: the 75%
    // diff that fails the default floor now passes — the floor is the single source
    // of truth, whole-tree or diff.
    let repo = TempRepo::new("cli-floor70");
    repo.write(
        "testing-conventions.toml",
        "[python.coverage]\nbranch = true\nfail_under = 70\n",
    );
    let base = baseline(&repo);
    repo.write("src/widget.py", WIDGET_PY_75);
    repo.write("src/widget_test.py", WIDGET_TEST_75);
    repo.commit("add a covered and an uncovered helper");

    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn a_coverage_exemption_lifts_a_below_floor_change() {
    // A line-scoped `coverage` exemption lifts the shim's changed lines from the
    // diff ratio — the same waiver the whole-tree floor honors, now line-scoped.
    let repo = TempRepo::new("exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"shim.py\"\nrules = [\"coverage\"]\n\
         lines = [\"1-3\"]\nreason = \"thin launcher; logic lives in tested modules\"\n",
    );
    let base = baseline(&repo);
    // The exemption `path` is scan-path-relative, so `shim.py` addresses `src/shim.py`.
    repo.write("src/shim.py", "def shim():\n    return 0\n    # noqa\n");
    repo.commit("add an untested launcher");

    // Flagged with no config…
    assert_eq!(run_coverage_base(&repo, &base, None).unwrap(), 1);
    // …and lifted by the `coverage` exemption.
    assert_eq!(
        run_coverage_base(&repo, &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}
