use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::changelog::{
    self, changed_packages, discover_layout, findings, fragment_name_ok, has_skip_line, is_exempt,
    migrations_enforced, Layout,
};

struct TempTree(PathBuf);

impl TempTree {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-changelog-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        TempTree(root)
    }

    fn dir(&self, rel: &str) -> &Self {
        std::fs::create_dir_all(self.0.join(rel)).unwrap();
        self
    }

    fn file(&self, rel: &str) -> &Self {
        std::fs::write(self.0.join(rel), "x\n").unwrap();
        self
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn owned(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| (*p).to_string()).collect()
}

fn per_package() -> Layout {
    Layout::PerPackage(vec!["packages".to_string()])
}

#[test]
fn fragment_dirs_under_packages_are_the_per_package_layout() {
    let tree = TempTree::new("per-pkg");
    tree.dir("packages/parser/changelog.d");
    assert_eq!(discover_layout(tree.path()), Some(per_package()));
}

#[test]
fn the_container_directories_are_discovered_not_assumed() {
    let tree = TempTree::new("containers");
    tree.dir("crates/parser/changelog.d");
    tree.dir("apps/site/changelog.d");
    assert_eq!(
        discover_layout(tree.path()),
        Some(Layout::PerPackage(vec![
            "apps".to_string(),
            "crates".to_string()
        ]))
    );
}

#[test]
fn a_fragment_dir_outside_any_package_is_the_pooled_layout() {
    let tree = TempTree::new("pooled");
    tree.dir("docs/changelog.d");
    assert_eq!(discover_layout(tree.path()), Some(Layout::Pooled));
}

#[test]
fn a_tree_with_no_fragment_dirs_has_no_layout_so_the_check_skips() {
    let tree = TempTree::new("none");
    tree.dir("packages/parser/src").file("README.md");
    assert_eq!(discover_layout(tree.path()), None);
}

#[test]
fn a_root_that_does_not_exist_has_no_layout() {
    let tree = TempTree::new("absent");
    assert_eq!(discover_layout(&tree.path().join("nowhere")), None);
}

#[test]
fn the_walk_skips_dependency_and_build_directories() {
    let tree = TempTree::new("skipped");
    tree.dir("node_modules/pkg/changelog.d");
    tree.dir("target/debug/changelog.d");
    tree.dir(".git/changelog.d");
    assert_eq!(discover_layout(tree.path()), None);
}

#[test]
fn migrations_are_enforced_only_where_the_repository_keeps_that_directory() {
    let with = TempTree::new("with-migrations");
    with.dir("packages/parser/changelog.d");
    with.dir("packages/parser/migrations.d");
    assert!(migrations_enforced(with.path()));

    let without = TempTree::new("without-migrations");
    without.dir("packages/parser/changelog.d");
    assert!(!migrations_enforced(without.path()));
}

#[test]
fn a_dated_kebab_case_name_is_well_formed() {
    assert!(fragment_name_ok("2026-09-21-drop-the-legacy-flag.md"));
}

#[test]
fn a_pooled_repository_writes_the_package_into_the_slug_and_the_name_still_parses() {
    assert!(fragment_name_ok(
        "2026-09-21-parser-drop-the-legacy-flag.md"
    ));
}

#[test]
fn a_name_without_the_date_prefix_is_malformed() {
    assert!(!fragment_name_ok("drop-the-legacy-flag.md"));
}

#[test]
fn a_name_that_is_not_markdown_or_is_too_short_for_a_slug_is_malformed() {
    assert!(!fragment_name_ok("2026-09-21-drop-the-flag.txt"));
    assert!(!fragment_name_ok("2026-09-21-.md"));
}

#[test]
fn a_name_carrying_uppercase_or_underscores_is_malformed() {
    assert!(!fragment_name_ok("2026-09-21-Drop_The_Flag.md"));
}

#[test]
fn a_skip_changelog_line_anywhere_in_a_body_bypasses_the_check() {
    assert!(has_skip_line(
        "refactor: rename the parser entry\n\nNo behavior change.\nskip-changelog: internal rename\n"
    ));
}

#[test]
fn a_body_with_no_skip_line_does_not_bypass() {
    assert!(!has_skip_line("feat: add the --strict flag\n"));
}

#[test]
fn the_frozen_archive_and_the_fragment_dirs_are_not_public_surface() {
    assert!(is_exempt("packages/parser/CHANGELOG.md", "packages/parser"));
    assert!(is_exempt(
        "packages/parser/MIGRATIONS.md",
        "packages/parser"
    ));
    assert!(is_exempt(
        "packages/parser/changelog.d/2026-09-21-a.md",
        "packages/parser"
    ));
}

#[test]
fn attestation_receipts_and_tests_are_not_public_surface() {
    assert!(is_exempt(
        "packages/parser/e2e-attestations/run.json",
        "packages/parser"
    ));
    assert!(is_exempt(
        "packages/parser/src/lex_test.py",
        "packages/parser"
    ));
    assert!(is_exempt(
        "packages/parser/tests/smoke.rs",
        "packages/parser"
    ));
    assert!(is_exempt(
        "packages/parser/src/lex.test.ts",
        "packages/parser"
    ));
}

#[test]
fn a_source_file_is_public_surface() {
    assert!(!is_exempt("packages/parser/src/lex.py", "packages/parser"));
}

#[test]
fn a_path_outside_the_package_is_never_that_package_s_exemption() {
    assert!(!is_exempt(
        "packages/emitter/CHANGELOG.md",
        "packages/parser"
    ));
}

#[test]
fn changed_packages_are_unique_and_sorted() {
    let changed = owned(&[
        "packages/parser/src/lex.py",
        "packages/parser/src/parse.py",
        "packages/emitter/src/emit.py",
        "README.md",
    ]);
    assert_eq!(
        changed_packages(&changed),
        vec![
            "packages/emitter".to_string(),
            "packages/parser".to_string()
        ]
    );
}

#[test]
fn a_package_that_changed_source_without_a_fragment_owes_both_kinds() {
    let changed = owned(&["packages/parser/src/lex.py"]);
    let found = findings(&per_package(), true, &changed, &[]);
    let messages: Vec<&str> = found.iter().map(|f| f.message.as_str()).collect();
    assert_eq!(found.len(), 2, "one finding per kind: {messages:?}");
    assert!(messages[0].contains("changelog"));
    assert!(messages[1].contains("migrations"));
}

#[test]
fn a_directory_outside_every_container_owes_nothing() {
    let changed = owned(&["docs/reference/checks/changelog.md"]);
    assert_eq!(findings(&per_package(), true, &changed, &[]), vec![]);
}

#[test]
fn a_package_carrying_both_fragments_owes_nothing() {
    let changed = owned(&[
        "packages/parser/src/lex.py",
        "packages/parser/changelog.d/2026-09-21-a.md",
        "packages/parser/migrations.d/2026-09-21-a.md",
    ]);
    let added = owned(&[
        "packages/parser/changelog.d/2026-09-21-a.md",
        "packages/parser/migrations.d/2026-09-21-a.md",
    ]);
    assert_eq!(findings(&per_package(), true, &changed, &added), vec![]);
}

#[test]
fn another_package_s_fragment_does_not_satisfy_this_one() {
    let changed = owned(&["packages/parser/src/lex.py"]);
    let added = owned(&["packages/emitter/changelog.d/2026-09-21-a.md"]);
    let found = findings(&per_package(), false, &changed, &added);
    assert_eq!(found.len(), 1);
}

#[test]
fn a_repository_keeping_no_migrations_owes_only_a_changelog_fragment() {
    let changed = owned(&["packages/parser/src/lex.py"]);
    let found = findings(&per_package(), false, &changed, &[]);
    assert_eq!(found.len(), 1);
    assert!(found[0].message.contains("changelog"));
}

#[test]
fn a_modified_fragment_does_not_satisfy_the_check_only_an_added_one_does() {
    let changed = owned(&[
        "packages/parser/src/lex.py",
        "packages/parser/changelog.d/2026-09-21-a.md",
    ]);
    let found = findings(&per_package(), false, &changed, &[]);
    assert_eq!(found.len(), 1, "a modified fragment is not an added one");
}

#[test]
fn a_package_that_changed_only_exempt_paths_owes_nothing() {
    let changed = owned(&["packages/parser/src/lex_test.py"]);
    assert_eq!(findings(&per_package(), true, &changed, &[]), vec![]);
}

#[test]
fn the_pooled_layout_asks_for_one_fragment_however_many_packages_changed() {
    let changed = owned(&["packages/parser/src/lex.py", "packages/emitter/src/emit.py"]);
    let found = findings(&Layout::Pooled, false, &changed, &[]);
    assert_eq!(
        found.len(),
        1,
        "repository-scoped: one fragment, not one per package"
    );
}

#[test]
fn the_pooled_layout_is_satisfied_by_a_single_root_fragment() {
    let changed = owned(&["packages/parser/src/lex.py"]);
    let added = owned(&["docs/changelog.d/2026-09-21-parser-drop-the-flag.md"]);
    assert_eq!(findings(&Layout::Pooled, false, &changed, &added), vec![]);
}

#[test]
fn the_pooled_layout_owes_nothing_when_only_exempt_paths_changed() {
    let changed = owned(&["packages/parser/src/lex_test.py"]);
    assert_eq!(findings(&Layout::Pooled, false, &changed, &[]), vec![]);
}

#[test]
fn a_malformed_fragment_name_is_reported_against_its_own_path() {
    let path = "packages/parser/changelog.d/drop-the-flag.md";
    let added = owned(&[path]);
    let found = findings(&per_package(), false, &added, &added);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].file.as_deref(), Some(path));
}

#[test]
fn a_fragment_directory_s_readme_is_not_an_entry() {
    let added = owned(&["packages/parser/changelog.d/README.md"]);
    assert_eq!(findings(&per_package(), false, &added, &added), vec![]);
}

#[test]
fn a_path_nested_below_a_fragment_directory_is_not_a_fragment() {
    let added = owned(&["packages/parser/changelog.d/drafts/drop-the-flag.md"]);
    let found = findings(&per_package(), false, &added, &added);
    assert_eq!(found, vec![], "nesting puts it outside the convention");
}

#[test]
fn a_fragment_outside_every_container_does_not_satisfy_a_package() {
    let changed = owned(&["packages/parser/src/lex.py"]);
    let added = owned(&["docs/changelog.d/2026-09-21-a.md"]);
    let found = findings(&per_package(), false, &changed, &added);
    assert_eq!(found.len(), 1);
}

#[test]
fn a_repository_path_that_does_not_exist_names_the_failing_read() {
    let tree = TempTree::new("no-repo");
    let err = changelog::changed_files(&tree.path().join("nowhere"), "HEAD").unwrap_err();
    assert!(
        format!("{err:#}").contains("running `git diff"),
        "names the read that failed: {err:#}"
    );
}

#[test]
fn the_kinds_vocabulary_reports_changelog_before_migrations() {
    assert_eq!(changelog::KINDS, ["changelog", "migrations"]);
}
