//! Integration tests for the unit-test location/naming check
//! (Python — issue #15; TypeScript — issue #18).
//!
//! A source file must have a *colocated* test named after it: `foo.py` →
//! `foo_test.py`, `foo-bar.ts` → `foo-bar.test.ts`. `missing_unit_tests`
//! returns the source files missing their twin; the `unit-location`
//! subcommand turns a non-empty result into a non-zero exit.
//!
//! Per the #3 guardrail, each language ships a clean fixture (every source
//! paired — must pass) and a red fixture (orphans present — must fail).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use testing_conventions::location::{missing_unit_tests, Language};
use testing_conventions::run;

/// Absolute path to a fixture tree under `tests/fixtures/unit_location/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_location")
        .join(name)
}

/// Orphans reported under `root` for `language`, as `/`-joined relative paths.
fn relative_orphans(root: &Path, language: Language) -> Vec<String> {
    missing_unit_tests(root, language)
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

/// Exit code of `unit-location [--lang <lang>] <fixture>`.
fn unit_location_exit(fixture_name: &str, lang: Option<&str>) -> i32 {
    let mut argv: Vec<OsString> = vec!["testing-conventions".into(), "unit-location".into()];
    if let Some(lang) = lang {
        argv.push("--lang".into());
        argv.push(lang.into());
    }
    argv.push(fixture(fixture_name).into_os_string());
    run(argv).expect("a readable tree should not error")
}

// ---- Python (#15) --------------------------------------------------------

#[test]
fn python_clean_tree_reports_no_orphans() {
    assert!(
        relative_orphans(&fixture("clean"), Language::Python).is_empty(),
        "every source in the clean tree has a colocated _test.py"
    );
}

#[test]
fn python_red_tree_reports_every_missing_twin() {
    assert_eq!(
        relative_orphans(&fixture("red"), Language::Python),
        vec!["lonely.py", "pkg/orphan.py"],
    );
}

#[test]
fn python_package_markers_are_not_orphans() {
    assert!(
        relative_orphans(&fixture("exempt"), Language::Python).is_empty(),
        "__init__.py is a package marker, never a unit-test subject"
    );
}

#[test]
fn python_missing_root_is_an_error() {
    assert!(
        missing_unit_tests(fixture("does_not_exist"), Language::Python).is_err(),
        "an unreadable root must be an error"
    );
}

#[test]
fn python_subcommand_exits_zero_on_a_clean_tree() {
    assert_eq!(unit_location_exit("clean", None), 0);
}

#[test]
fn python_subcommand_exits_nonzero_on_a_red_tree() {
    assert_eq!(unit_location_exit("red", None), 1);
}

// ---- TypeScript (#18) ----------------------------------------------------

#[test]
fn typescript_clean_tree_reports_no_orphans() {
    // The clean tree also holds a `*.d.ts` with no twin; it must be ignored.
    assert!(
        relative_orphans(&fixture("typescript/clean"), Language::TypeScript).is_empty(),
        "every .ts/.tsx source is paired; declaration files are ignored"
    );
}

#[test]
fn typescript_red_tree_reports_every_missing_twin() {
    assert_eq!(
        relative_orphans(&fixture("typescript/red"), Language::TypeScript),
        vec!["lonely.ts", "pkg/orphan.ts"],
    );
}

#[test]
fn typescript_subcommand_exits_zero_on_a_clean_tree() {
    assert_eq!(
        unit_location_exit("typescript/clean", Some("typescript")),
        0
    );
}

#[test]
fn typescript_subcommand_exits_nonzero_on_a_red_tree() {
    assert_eq!(unit_location_exit("typescript/red", Some("typescript")), 1);
}
