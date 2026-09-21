use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-changelog-e2e-{}-{}-{}",
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

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git should run");
    assert!(status.success(), "git {args:?} failed");
}

/// A repo with the per-package layout and one committed fragment, so the layout is discoverable.
fn seeded(slug: &str) -> (TempRepo, String) {
    let repo = TempRepo::new(slug);
    repo.write("packages/parser/src/lex.py", "def lex():\n    pass\n");
    repo.write("packages/parser/changelog.d/2026-01-01-seed.md", "seed\n");
    repo.write("packages/parser/migrations.d/2026-01-01-seed.md", "seed\n");
    repo.commit("seed");
    let base = repo.head();
    (repo, base)
}

#[test]
fn changing_package_source_without_a_fragment_exits_nonzero() {
    let (repo, base) = seeded("missing");
    repo.write("packages/parser/src/lex.py", "def lex():\n    return 1\n");
    repo.commit("feat: lex returns");

    let out = repo.changelog(&base);
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("::error::"),
        "annotation expected: {stdout}"
    );
    assert!(
        stdout.contains("packages/parser"),
        "names the package: {stdout}"
    );
}

#[test]
fn changing_package_source_with_both_fragments_exits_zero() {
    let (repo, base) = seeded("satisfied");
    repo.write("packages/parser/src/lex.py", "def lex():\n    return 1\n");
    repo.write(
        "packages/parser/changelog.d/2026-09-21-lex-returns.md",
        "x\n",
    );
    repo.write(
        "packages/parser/migrations.d/2026-09-21-lex-returns.md",
        "x\n",
    );
    repo.commit("feat: lex returns");

    let out = repo.changelog(&base);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn a_skip_changelog_line_on_any_commit_exits_zero() {
    let (repo, base) = seeded("skip");
    repo.write("packages/parser/src/lex.py", "def lex():\n    return 1\n");
    repo.commit("refactor: rename\n\nskip-changelog: internal rename");

    let out = repo.changelog(&base);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn a_repository_keeping_no_fragment_directories_skips_and_exits_zero() {
    let repo = TempRepo::new("no-fragments");
    repo.write("packages/parser/src/lex.py", "def lex():\n    pass\n");
    repo.commit("seed");
    let base = repo.head();
    repo.write("packages/parser/src/lex.py", "def lex():\n    return 1\n");
    repo.commit("feat: lex returns");

    let out = repo.changelog(&base);
    assert_eq!(
        out.status.code(),
        Some(0),
        "a repo that keeps no fragments is skipped, never failed"
    );
}

#[test]
fn changing_only_a_test_file_exits_zero() {
    let (repo, base) = seeded("tests-only");
    repo.write(
        "packages/parser/src/lex_test.py",
        "def test_lex():\n    pass\n",
    );
    repo.commit("test: cover lex");

    let out = repo.changelog(&base);
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn a_malformed_fragment_name_exits_nonzero_and_annotates_the_file() {
    let (repo, base) = seeded("malformed");
    repo.write("packages/parser/src/lex.py", "def lex():\n    return 1\n");
    repo.write("packages/parser/changelog.d/lex-returns.md", "x\n");
    repo.write(
        "packages/parser/migrations.d/2026-09-21-lex-returns.md",
        "x\n",
    );
    repo.commit("feat: lex returns");

    let out = repo.changelog(&base);
    assert_eq!(out.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("::error file=packages/parser/changelog.d/lex-returns.md::"),
        "annotates the offending file: {stdout}"
    );
}
