//! The `changelog` subcommand end to end over a real repository.
//!
//! `changelog::*` reads git directly and takes no injection point, so proving that a scope
//! owing a fragment exits `1` — and that the same tree with a `skip-changelog:` trailer exits
//! `0` — needs a real repository. That fixture is effectful by nature, which is why these
//! cases live out here rather than beside the dispatcher in `src/lib.rs`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-changelog-run-{}-{}-{}",
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

/// A per-package tree whose `parser` scope has uncommitted code changes since the returned
/// base, with no fragment recording them.
fn owing_a_fragment(slug: &str) -> (TempRepo, String) {
    let repo = TempRepo::new(slug);
    repo.write("packages/parser/changelog.d/.gitkeep", "")
        .write("packages/parser/src/parse.ts", "export const a = 1;\n");
    let base = repo.commit("baseline");
    repo.write("packages/parser/src/parse.ts", "export const a = 2;\n");
    (repo, base)
}

#[test]
fn a_scope_that_changed_code_without_a_fragment_fails() {
    let (repo, base) = owing_a_fragment("owed");
    repo.commit("feat(parser): change public surface");
    assert_eq!(changelog(&base, repo.path()), 1);
}

#[test]
fn a_skip_changelog_line_bypasses_the_check_the_same_tree_would_fail() {
    let (repo, base) = owing_a_fragment("skip");
    repo.commit("refactor(parser): rename a local\n\nskip-changelog: no public surface");
    assert_eq!(changelog(&base, repo.path()), 0);
}

#[test]
fn a_scope_that_added_its_fragment_passes() {
    let (repo, base) = owing_a_fragment("paid");
    repo.write(
        "packages/parser/changelog.d/2026-10-05-change-a.md",
        "changed a\n",
    );
    repo.commit("feat(parser): change public surface, with its fragment");
    assert_eq!(changelog(&base, repo.path()), 0);
}

/// A per-package tree keeping both fragment directories, whose `parser` scope has uncommitted
/// code changes since the returned base.
fn owing_with_migrations(slug: &str) -> (TempRepo, String) {
    let repo = TempRepo::new(slug);
    repo.write("packages/parser/changelog.d/.gitkeep", "")
        .write("packages/parser/migrations.d/.gitkeep", "")
        .write("packages/parser/src/parse.ts", "export const a = 1;\n");
    let base = repo.commit("baseline");
    repo.write("packages/parser/src/parse.ts", "export const a = 2;\n");
    (repo, base)
}

#[test]
fn a_changelog_fragment_alone_pays_when_no_commit_calls_the_change_breaking() {
    let (repo, base) = owing_with_migrations("not-breaking");
    repo.write(
        "packages/parser/changelog.d/2026-10-06-change-a.md",
        "changed a\n",
    );
    repo.commit("feat(parser): change public surface");
    assert_eq!(
        changelog(&base, repo.path()),
        0,
        "a repository keeping migrations.d owes a migration for a breaking change alone"
    );
}

#[test]
fn a_breaking_line_requires_a_migrations_fragment() {
    let (repo, base) = owing_with_migrations("breaking");
    repo.write(
        "packages/parser/changelog.d/2026-10-06-change-a.md",
        "changed a\n",
    );
    repo.commit("feat(parser)!: drop the legacy flag\n\nbreaking: the --legacy flag is gone");
    assert_eq!(changelog(&base, repo.path()), 1);
}

#[test]
fn a_breaking_line_is_paid_by_both_fragments() {
    let (repo, base) = owing_with_migrations("breaking-paid");
    repo.write(
        "packages/parser/changelog.d/2026-10-06-change-a.md",
        "changed a\n",
    );
    repo.write(
        "packages/parser/migrations.d/2026-10-06-change-a.md",
        "migrate a\n",
    );
    repo.commit("feat(parser)!: drop the legacy flag\n\nbreaking: the --legacy flag is gone");
    assert_eq!(changelog(&base, repo.path()), 0);
}

#[test]
fn a_migrations_fragment_no_commit_asked_for_is_allowed() {
    let (repo, base) = owing_with_migrations("unsolicited");
    repo.write(
        "packages/parser/changelog.d/2026-10-06-change-a.md",
        "changed a\n",
    );
    repo.write(
        "packages/parser/migrations.d/2026-10-06-change-a.md",
        "migrate a\n",
    );
    repo.commit("feat(parser): change public surface");
    assert_eq!(
        changelog(&base, repo.path()),
        0,
        "the `breaking:` line makes a migrations fragment required, never forbidden"
    );
}
