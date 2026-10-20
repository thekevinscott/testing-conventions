use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-wl-scope-e2e-{}-{}-{}",
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

    /// `testing-conventions workflow-lint scope <args>` run at the repo root.
    fn scope(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
            .args(["workflow-lint", "scope"])
            .args(args)
            .current_dir(&self.0)
            .output()
            .expect("the built binary should run")
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

fn seeded(slug: &str) -> (TempRepo, String) {
    let repo = TempRepo::new(slug);
    repo.write(".github/workflows/ci.yml", "on: push\n");
    repo.write("packages/lib/src/lib.py", "x = 1\n");
    repo.commit("seed");
    let base = repo.head();
    (repo, base)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_changed_workflow_prints_true_and_exits_zero() {
    let (repo, base) = seeded("changed");
    repo.write(".github/workflows/ci.yml", "on: pull_request\n");
    repo.commit("ci: change trigger");

    let out = repo.scope(&["--base", &base]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "true\n");
}

#[test]
fn a_source_only_change_prints_false_and_exits_zero() {
    let (repo, base) = seeded("source");
    repo.write("packages/lib/src/lib.py", "x = 2\n");
    repo.commit("feat: bump");

    let out = repo.scope(&["--base", &base]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "false\n");
}

#[test]
fn a_named_path_scopes_the_read() {
    let (repo, base) = seeded("named");
    repo.write("ci/workflows/build.yml", "on: push\n");
    repo.commit("ci: add build under ci/");

    assert_eq!(stdout(&repo.scope(&["--base", &base, "ci"])), "true\n");
    assert_eq!(stdout(&repo.scope(&["--base", &base])), "false\n");
}

#[test]
fn no_base_prints_true() {
    let (repo, _) = seeded("no-base");
    let out = repo.scope(&[]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "true\n");
}

#[test]
fn an_unreadable_diff_prints_true_and_names_the_reason() {
    let (repo, _) = seeded("bad-base");
    let out = repo.scope(&["--base", "no-such-ref"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(stdout(&out), "true\n");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no-such-ref"), "names the reason: {stderr}");
}

#[test]
fn the_lint_still_runs_without_the_subcommand() {
    let (repo, _) = seeded("lint");
    repo.write(
        ".github/workflows/ci.yml",
        "on: push\njobs:\n  a:\n    runs-on: x\n    steps:\n      - run: |\n          for f in *; do echo $f; done\n",
    );
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("workflow-lint")
        .current_dir(&repo.0)
        .output()
        .expect("the built binary should run");
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
}
