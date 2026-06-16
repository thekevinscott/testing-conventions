//! Integration tests for the workflow guard (#92): exercise the library API over
//! fixture workflow files. The guard: the reusable workflow's `testing-conventions`
//! invocations must name subcommands the binary still exposes, so a rename like
//! `unit location` → `unit colocated-test` (#55) can't strand the `@v0` path.
//!
//! The e2e suite (`workflow_e2e.rs`) drives the same fixtures through the built
//! binary; this checks the library contract directly, against the binary's own
//! command tree (`testing_conventions::command()`).

use std::path::PathBuf;

use testing_conventions::{command, workflow};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/workflow")
        .join(name)
}

#[test]
fn clean_workflow_has_no_violations() {
    let violations = workflow::check(fixture("clean"), &command()).unwrap();
    assert!(
        violations.is_empty(),
        "unexpected violations: {violations:?}"
    );
}

#[test]
fn red_flags_the_renamed_subcommand() {
    let violations = workflow::check(fixture("red"), &command()).unwrap();
    // `unit location` (renamed to `unit colocated-test`, #55) is on line 9.
    assert!(
        violations
            .iter()
            .any(|v| v.line == 9 && v.message.contains("location")),
        "expected a violation naming `location` on line 9: {violations:?}"
    );
}

#[test]
fn red_flags_the_old_flat_form() {
    let violations = workflow::check(fixture("red"), &command()).unwrap();
    // The pre-#22 flat `unit-location` is gone entirely; it sits on line 11.
    assert!(
        violations
            .iter()
            .any(|v| v.line == 11 && v.message.contains("unit-location")),
        "expected a violation naming `unit-location` on line 11: {violations:?}"
    );
}

#[test]
fn red_flags_every_stranded_invocation() {
    let violations = workflow::check(fixture("red"), &command()).unwrap();
    assert_eq!(
        violations.len(),
        2,
        "both stranded invocations should be flagged: {violations:?}"
    );
}

#[test]
fn invocations_are_extracted_from_the_shell() {
    // Extraction is the implemented half: it finds the calls — version pin, flags,
    // and all — regardless of whether the subcommand still exists.
    let found = workflow::invocations(fixture("red")).unwrap();
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].args.first().map(String::as_str), Some("unit"));
}
