//! The `changelog` check over the pooled fragment layout: one `docs/changelog.d/` (plus
//! `docs/migrations.d/`) for every package, fragments named `YYYY-MM-DD-<pkg>-<slug>.md`.
//!
//! The pooled rules read package roots off the filesystem and the diff out of git, so the cases
//! that distinguish them need a real repository rather than a pure-function call.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-changelog-pooled-{}-{}-{}",
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

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn changelog(base: &str, root: &Path) -> i32 {
    testing_conventions::run([
        "testing-conventions".as_ref(),
        "changelog".as_ref(),
        "--base".as_ref(),
        base.as_ref(),
        root.as_os_str(),
    ])
    .expect("the changelog check should run")
}

/// A pooled tree: one fragment directory under `docs/`, one package root at `packages/foo`, and
/// documentation outside every package.
fn pooled(slug: &str) -> (TempRepo, String) {
    let repo = TempRepo::new(slug);
    repo.write("docs/changelog.d/2026-01-01-foo-seed.md", "seed\n")
        .write("docs/guide.md", "# guide\n")
        .write("README.md", "# readme\n")
        .write("packages/foo/package.json", "{\"name\": \"foo\"}\n")
        .write("packages/foo/src/a.py", "def a():\n    pass\n");
    let base = repo.commit("seed");
    (repo, base)
}

#[test]
fn a_readme_edit_outside_every_package_owes_nothing() {
    let (repo, base) = pooled("readme");
    repo.write("README.md", "# readme, edited\n");
    repo.commit("docs: edit the readme");
    assert_eq!(
        changelog(&base, repo.path()),
        0,
        "a path under no package root is not public surface"
    );
}

#[test]
fn a_docs_edit_outside_every_package_owes_nothing() {
    let (repo, base) = pooled("docs");
    repo.write("docs/guide.md", "# guide, edited\n");
    repo.commit("docs: edit the guide");
    assert_eq!(changelog(&base, repo.path()), 0);
}

#[test]
fn a_package_source_change_with_no_fragment_fails() {
    let (repo, base) = pooled("unpaid");
    repo.write("packages/foo/src/a.py", "def a():\n    return 1\n");
    repo.commit("feat(foo): a returns");
    assert_eq!(changelog(&base, repo.path()), 1);
}

#[test]
fn a_fragment_naming_the_changed_package_pays() {
    let (repo, base) = pooled("paid");
    repo.write("packages/foo/src/a.py", "def a():\n    return 1\n")
        .write("docs/changelog.d/2026-10-06-foo-a-returns.md", "x\n");
    repo.commit("feat(foo): a returns");
    assert_eq!(changelog(&base, repo.path()), 0);
}

#[test]
fn a_fragment_naming_another_package_does_not_pay_for_this_one() {
    let (repo, base) = pooled("misnamed");
    repo.write("packages/foo/src/a.py", "def a():\n    return 1\n")
        .write("docs/changelog.d/2026-10-06-bar-a-returns.md", "x\n");
    repo.commit("feat(foo): a returns");
    assert_eq!(
        changelog(&base, repo.path()),
        1,
        "the <pkg> segment names the package whose surface changed"
    );
}

#[test]
fn a_migrations_fragment_naming_another_package_does_not_pay_for_this_one() {
    let (repo, base) = pooled("misnamed-migration");
    repo.write("packages/foo/src/a.py", "def a():\n    return 1\n")
        .write("docs/changelog.d/2026-10-06-foo-a-returns.md", "x\n")
        .write("docs/migrations.d/2026-10-06-bar-a-returns.md", "x\n");
    repo.commit("feat(foo)!: a returns\n\nbreaking: a returns a value now");
    assert_eq!(
        changelog(&base, repo.path()),
        1,
        "both fragment kinds name the package whose surface changed"
    );
}

#[test]
fn a_tree_with_no_package_roots_treats_its_whole_surface_as_one_package() {
    let repo = TempRepo::new("no-packages");
    repo.write("docs/changelog.d/2026-01-01-seed.md", "seed\n")
        .write("src/a.py", "def a():\n    pass\n");
    let base = repo.commit("seed");
    repo.write("src/a.py", "def a():\n    return 1\n");
    repo.commit("feat: a returns");
    assert_eq!(
        changelog(&base, repo.path()),
        1,
        "with no package root to scope to, every non-exempt path is surface"
    );
}

#[test]
fn a_per_package_fragment_need_not_name_its_package() {
    let repo = TempRepo::new("per-package");
    repo.write("packages/foo/package.json", "{\"name\": \"foo\"}\n")
        .write("packages/foo/changelog.d/2026-01-01-seed.md", "seed\n")
        .write("packages/foo/src/a.py", "def a():\n    pass\n");
    let base = repo.commit("seed");
    repo.write("packages/foo/src/a.py", "def a():\n    return 1\n")
        .write("packages/foo/changelog.d/2026-10-06-a-returns.md", "x\n");
    repo.commit("feat(foo): a returns");
    assert_eq!(
        changelog(&base, repo.path()),
        0,
        "the directory names the package, so the filename does not have to"
    );
}
