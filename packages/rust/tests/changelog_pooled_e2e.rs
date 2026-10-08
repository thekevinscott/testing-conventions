//! The shipped binary against a pooled fragment layout, through its own process and exit code.

use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-changelog-pooled-e2e-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let repo = TempRepo(root);
        repo.git(&["init", "-q", "-b", "main"]);
        repo.git(&["config", "user.email", "test@example.com"]);
        repo.git(&["config", "user.name", "Test"]);
        repo
    }

    fn git(&self, args: &[&str]) {
        let status = Command::new("git")
            .args(args)
            .current_dir(&self.0)
            .status()
            .expect("git should run");
        assert!(status.success(), "git {args:?} failed");
    }

    fn write(&self, rel: &str, contents: &str) -> &Self {
        let full = self.0.join(rel);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, contents).unwrap();
        self
    }

    fn commit(&self, message: &str) -> String {
        self.git(&["add", "-A"]);
        self.git(&["-c", "commit.gpgsign=false", "commit", "-q", "-m", message]);
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// `testing-conventions changelog --base <base>` run at the repo root.
    fn changelog(&self, base: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
            .args(["changelog", "--base", base])
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

/// A pooled tree: one fragment directory under `docs/`, one package root at `packages/foo`, and a
/// readme outside every package.
fn pooled(slug: &str) -> (TempRepo, String) {
    let repo = TempRepo::new(slug);
    repo.write("docs/changelog.d/2026-01-01-foo-seed.md", "seed\n")
        .write("README.md", "# readme\n")
        .write("packages/foo/package.json", "{\"name\": \"foo\"}\n")
        .write("packages/foo/src/a.py", "def a():\n    pass\n");
    let base = repo.commit("seed");
    (repo, base)
}

fn stdout_of(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn a_readme_edit_outside_every_package_exits_zero() {
    let (repo, base) = pooled("readme");
    repo.write("README.md", "# readme, edited\n");
    repo.commit("docs: edit the readme");

    let out = repo.changelog(&base);
    assert_eq!(out.status.code(), Some(0), "{}", stdout_of(&out));
}

#[test]
fn a_fragment_naming_another_package_exits_nonzero_and_names_the_changed_one() {
    let (repo, base) = pooled("misnamed");
    repo.write("packages/foo/src/a.py", "def a():\n    return 1\n")
        .write("docs/changelog.d/2026-10-06-bar-a-returns.md", "x\n");
    repo.commit("feat(foo): a returns");

    let out = repo.changelog(&base);
    assert_eq!(out.status.code(), Some(1));
    let stdout = stdout_of(&out);
    assert!(
        stdout.contains("packages/foo"),
        "names the package whose surface changed: {stdout}"
    );
    assert!(
        stdout.contains("YYYY-MM-DD-foo-<slug>.md"),
        "names the fragment that would pay: {stdout}"
    );
}

#[test]
fn a_fragment_naming_the_changed_package_exits_zero() {
    let (repo, base) = pooled("paid");
    repo.write("packages/foo/src/a.py", "def a():\n    return 1\n")
        .write("docs/changelog.d/2026-10-06-foo-a-returns.md", "x\n");
    repo.commit("feat(foo): a returns");

    let out = repo.changelog(&base);
    assert_eq!(out.status.code(), Some(0), "{}", stdout_of(&out));
}
