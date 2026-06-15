//! Integration tests for the Python unit-test location/naming check (issue #15).
//!
//! These pin the contract from #15: a Python source file `foo.py` must have a
//! colocated `foo_test.py`. `missing_unit_tests` walks a directory and returns
//! every source file missing its twin; the `unit-location` subcommand turns a
//! non-empty result into a non-zero exit.
//!
//! Per the #3 guardrail, the check ships a clean fixture
//! (`unit_location/clean`, every source paired — must pass) and a red fixture
//! (`unit_location/red`, two orphans — must fail).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use testing_conventions::location::missing_unit_tests;
use testing_conventions::run;

/// Absolute path to a fixture tree under `tests/fixtures/unit_location/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_location")
        .join(name)
}

/// Orphans reported under `root`, as `/`-joined paths relative to it.
fn relative_orphans(root: &Path) -> Vec<String> {
    missing_unit_tests(root)
        .expect("walking a readable tree should succeed")
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .expect("an orphan must live under the scanned root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

/// Invoke the `unit-location <fixture>` subcommand, returning its exit code.
fn run_unit_location(fixture_name: &str) -> i32 {
    let args: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit-location".into(),
        fixture(fixture_name).into_os_string(),
    ];
    run(args).expect("a readable tree should not error")
}

#[test]
fn clean_tree_reports_no_orphans() {
    assert!(
        relative_orphans(&fixture("clean")).is_empty(),
        "every source in the clean tree has a colocated _test.py"
    );
}

#[test]
fn red_tree_reports_every_missing_twin() {
    assert_eq!(
        relative_orphans(&fixture("red")),
        vec!["lonely.py", "pkg/orphan.py"],
    );
}

#[test]
fn package_markers_are_not_orphans() {
    assert!(
        relative_orphans(&fixture("exempt")).is_empty(),
        "__init__.py is a package marker, never a unit-test subject"
    );
}

#[test]
fn a_missing_root_is_an_error() {
    let result = missing_unit_tests(fixture("does_not_exist"));
    assert!(
        result.is_err(),
        "an unreadable root must be an error, got: {result:?}"
    );
}

#[test]
fn subcommand_exits_zero_on_a_clean_tree() {
    assert_eq!(run_unit_location("clean"), 0);
}

#[test]
fn subcommand_exits_nonzero_on_a_red_tree() {
    assert_eq!(run_unit_location("red"), 1);
}
