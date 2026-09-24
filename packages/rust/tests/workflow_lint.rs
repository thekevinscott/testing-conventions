use std::path::PathBuf;

use testing_conventions::workflow_lint;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workflow_lint")
        .join(name)
}

#[test]
fn a_clean_tree_has_no_findings() {
    let findings = workflow_lint::scan(fixture("clean")).unwrap();
    assert!(findings.is_empty(), "unexpected findings: {findings:?}");
}

#[test]
fn discovery_skips_the_yaml_github_never_reads_as_ci() {
    // Both decoys carry a `for` loop: a test fixture under the scanned root, and a file nested a
    // directory below `workflows/`. A scan that read either would report it.
    let findings = workflow_lint::scan(fixture("clean")).unwrap();
    assert_eq!(findings.len(), 0, "a decoy was scanned: {findings:?}");
}

#[test]
fn a_red_tree_reports_the_workflow_and_the_composite_action() {
    let findings = workflow_lint::scan(fixture("red")).unwrap();
    let steps: Vec<&str> = findings.iter().map(|f| f.step.as_str()).collect();
    assert_eq!(
        steps,
        vec!["Rewrite the version", "Publish bootstrap stubs"]
    );
}

#[test]
fn a_finding_names_its_file_line_and_reasons() {
    let findings = workflow_lint::scan(fixture("red/workflows/publish.yml")).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 13, "the body starts on line 13");
    assert_eq!(findings[0].kind, "run");
    assert_eq!(findings[0].reasons, vec!["for loop"]);
    assert!(findings[0].file.ends_with("workflows/publish.yml"));
}

#[test]
fn a_named_file_is_scanned_whatever_its_name() {
    // The decoy is not CI by discovery, but naming it directly scans it — so a caller can point
    // the check at any file.
    let findings = workflow_lint::scan(fixture("clean/selftest/workflow-fixture.yaml")).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].reasons, vec!["for loop"]);
}

#[test]
fn a_repository_with_no_ci_directory_is_not_an_error() {
    let findings = workflow_lint::scan(fixture("no-such-directory")).unwrap();
    assert!(findings.is_empty());
}

fn unique_tmp(slug: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "tc-workflow-lint-{slug}-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn an_unreadable_workflow_names_the_file() {
    let dir = unique_tmp("nonutf8");
    let file = dir.join("ci.yml");
    std::fs::write(&file, [0xFF, 0xFE, 0x00]).unwrap();
    let err = workflow_lint::scan(&file).unwrap_err();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        format!("{err:#}").contains("reading workflow"),
        "got: {err:#}"
    );
}
