use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::workflow_lint::in_scope;

struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-wl-scope-{}-{}-{}",
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

    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.0.join(rel)).unwrap();
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

    fn in_scope(&self, path: &str, base: Option<&str>) -> anyhow::Result<bool> {
        in_scope(&self.0, Path::new(path), base)
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

/// A repo holding a workflow, a composite action, a non-CI YAML under `.github`, and package
/// source, with HEAD as the returned base.
fn seeded(slug: &str) -> (TempRepo, String) {
    let repo = TempRepo::new(slug);
    repo.write(".github/workflows/ci.yml", "on: push\n");
    repo.write(".github/actions/setup/action.yml", "runs: {}\n");
    repo.write(".github/dependabot.yml", "version: 2\n");
    repo.write("packages/lib/src/lib.py", "x = 1\n");
    repo.commit("seed");
    let base = repo.head();
    (repo, base)
}

#[test]
fn a_changed_workflow_is_in_scope() {
    let (repo, base) = seeded("workflow");
    repo.write(".github/workflows/ci.yml", "on: pull_request\n");
    repo.commit("ci: change trigger");
    assert!(repo.in_scope(".github", Some(&base)).unwrap());
}

#[test]
fn an_added_workflow_is_in_scope() {
    let (repo, base) = seeded("added");
    repo.write(".github/workflows/release.yaml", "on: push\n");
    repo.commit("ci: add release");
    assert!(repo.in_scope(".github", Some(&base)).unwrap());
}

#[test]
fn a_deleted_workflow_is_in_scope() {
    let (repo, base) = seeded("deleted");
    repo.remove(".github/workflows/ci.yml");
    repo.commit("ci: drop ci");
    assert!(repo.in_scope(".github", Some(&base)).unwrap());
}

#[test]
fn a_changed_composite_action_is_in_scope() {
    let (repo, base) = seeded("action");
    repo.write(
        ".github/actions/setup/action.yml",
        "runs: {using: composite}\n",
    );
    repo.commit("ci: change action");
    assert!(repo.in_scope(".github", Some(&base)).unwrap());
}

#[test]
fn a_change_to_package_source_alone_is_out_of_scope() {
    let (repo, base) = seeded("source");
    repo.write("packages/lib/src/lib.py", "x = 2\n");
    repo.commit("feat: bump");
    assert!(!repo.in_scope(".github", Some(&base)).unwrap());
}

#[test]
fn a_yaml_file_github_does_not_execute_is_out_of_scope() {
    let (repo, base) = seeded("dependabot");
    repo.write(".github/dependabot.yml", "version: 2\nupdates: []\n");
    repo.write(".github/workflows/fixtures/nested.yml", "on: push\n");
    repo.commit("chore: dependabot and a nested fixture");
    assert!(!repo.in_scope(".github", Some(&base)).unwrap());
}

#[test]
fn a_workflow_outside_the_scanned_path_is_out_of_scope() {
    let (repo, base) = seeded("outside");
    repo.write("examples/.github/workflows/demo.yml", "on: push\n");
    repo.commit("docs: example workflow");
    assert!(!repo.in_scope(".github", Some(&base)).unwrap());
}

#[test]
fn a_named_file_is_in_scope_when_it_changes() {
    let (repo, base) = seeded("named");
    repo.write("action.yml", "runs: {}\n");
    repo.commit("seed root action");
    let base_with_action = repo.head();
    repo.write("action.yml", "runs: {using: composite}\n");
    repo.commit("ci: change root action");
    assert!(repo
        .in_scope("action.yml", Some(&base_with_action))
        .unwrap());
    assert!(repo.in_scope("action.yml", Some(&base)).unwrap());
}

#[test]
fn a_named_file_is_out_of_scope_when_only_other_files_change() {
    let (repo, base) = seeded("named-untouched");
    repo.write(".github/workflows/ci.yml", "on: pull_request\n");
    repo.commit("ci: change trigger");
    assert!(!repo
        .in_scope(".github/actions/setup/action.yml", Some(&base))
        .unwrap());
}

#[test]
fn no_base_puts_the_whole_tree_in_scope() {
    let (repo, _) = seeded("no-base");
    assert!(repo.in_scope(".github", None).unwrap());
}

#[test]
fn a_base_that_names_no_commit_is_an_error() {
    let (repo, _) = seeded("bad-base");
    let err = repo
        .in_scope(".github", Some("no-such-ref"))
        .expect_err("an unresolvable base cannot be diffed");
    assert!(format!("{err:#}").contains("no-such-ref"), "{err:#}");
}
