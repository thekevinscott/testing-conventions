//! Integration tests for the unit `colocated-test` check (Python — issue #15;
//! TypeScript — issue #18; exemptions — issue #32; renamed from `location` in #55).
//!
//! A source file must have a *colocated* test named after it: `foo.py` →
//! `foo_test.py`, `foo-bar.ts` → `foo-bar.test.ts`. `missing_unit_tests`
//! returns the source files missing their twin; the `unit colocated-test`
//! subcommand turns a non-empty result into a non-zero exit.
//!
//! Empty/comment-only files are never subjects (no logic to test), and a file
//! listed in the config `exempt` table is a deliberate, reason-required
//! omission. A stale exempt entry (path that no longer exists) is an error.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use testing_conventions::colocated_test::{missing_unit_tests, Language};
use testing_conventions::run;

/// Absolute path to a fixture tree under `tests/fixtures/colocated_test/`.
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/colocated_test")
        .join(name)
}

/// Orphans reported under `root` for `language` (no exemptions), as `/`-joined
/// relative paths.
fn relative_orphans(root: &Path, language: Language) -> Vec<String> {
    orphans_with(root, language, &BTreeSet::new())
}

/// Orphans reported under `root` for `language` with `exempt` paths lifted.
fn orphans_with(root: &Path, language: Language, exempt: &BTreeSet<String>) -> Vec<String> {
    missing_unit_tests(root, language, exempt)
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

/// A `BTreeSet` of exempt relative paths.
fn exempt(paths: &[&str]) -> BTreeSet<String> {
    paths.iter().map(|p| p.to_string()).collect()
}

/// Result of `unit colocated-test --language <lang> --config <fixture>/testing-conventions.toml
/// <fixture>`. The config is co-located with each fixture; for trees without one
/// (clean/red), the absent file simply means "no exemptions".
fn unit_colocated_test_run(fixture_name: &str, language: &str) -> anyhow::Result<i32> {
    let dir = fixture(fixture_name);
    let config = dir.join("testing-conventions.toml");
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "colocated-test".into(),
        "--language".into(),
        language.into(),
        "--config".into(),
        config.into_os_string(),
        dir.into_os_string(),
    ];
    run(argv)
}

/// Exit code of `unit colocated-test` over `fixture_name`.
fn unit_colocated_test_exit(fixture_name: &str, language: &str) -> i32 {
    unit_colocated_test_run(fixture_name, language).expect("a readable tree should not error")
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
fn python_missing_root_is_an_error() {
    assert!(
        missing_unit_tests(
            fixture("does_not_exist"),
            Language::Python,
            &BTreeSet::new()
        )
        .is_err(),
        "an unreadable root must be an error"
    );
}

#[test]
fn python_subcommand_exits_zero_on_a_clean_tree() {
    assert_eq!(unit_colocated_test_exit("clean", "python"), 0);
}

#[test]
fn python_subcommand_exits_nonzero_on_a_red_tree() {
    assert_eq!(unit_colocated_test_exit("red", "python"), 1);
}

// ---- conftest.py is pytest support, never a subject (#112) ---------------

#[test]
fn python_conftest_is_a_non_subject() {
    // conftest.py holds pytest fixtures — test support, not a unit under test
    // (there is no `conftest_test.py`). With widget.py paired, conftest.py is the
    // only file that could be flagged; it must not be reported as an orphan.
    assert!(
        relative_orphans(&fixture("python_conftest"), Language::Python).is_empty(),
        "conftest.py is pytest support, never a colocated-test subject"
    );
}

#[test]
fn python_conftest_subcommand_exits_zero() {
    assert_eq!(unit_colocated_test_exit("python_conftest", "python"), 0);
}

// ---- TypeScript (#18) ----------------------------------------------------

#[test]
fn typescript_clean_tree_reports_no_orphans() {
    // The clean tree pairs .ts/.tsx/.mts/.cts sources and holds `*.d.ts` /
    // `*.d.mts` declarations with no twin; declarations must be ignored.
    assert!(
        relative_orphans(&fixture("typescript/clean"), Language::TypeScript).is_empty(),
        "every .ts/.tsx/.mts/.cts source is paired; declaration files are ignored"
    );
}

#[test]
fn typescript_red_tree_reports_every_missing_twin() {
    assert_eq!(
        relative_orphans(&fixture("typescript/red"), Language::TypeScript),
        vec!["bridge.cts", "lonely.ts", "pkg/orphan.ts", "solo.mts"],
    );
}

#[test]
fn typescript_subcommand_exits_zero_on_a_clean_tree() {
    assert_eq!(
        unit_colocated_test_exit("typescript/clean", "typescript"),
        0
    );
}

#[test]
fn typescript_subcommand_exits_nonzero_on_a_red_tree() {
    assert_eq!(unit_colocated_test_exit("typescript/red", "typescript"), 1);
}

// ---- Exemptions (#32) ----------------------------------------------------

#[test]
fn empty_init_is_a_non_subject_but_content_and_shims_are_orphans() {
    // In `python_exempt`: the empty top-level __init__.py is a non-subject (no
    // code), core.py is paired, and pkg/__init__.py (has code) + cli.py are
    // orphans absent any exemption — __init__.py is no longer auto-exempt.
    assert_eq!(
        relative_orphans(&fixture("python_exempt"), Language::Python),
        vec!["cli.py", "pkg/__init__.py"],
    );
}

#[test]
fn config_exemptions_lift_listed_files() {
    // Lifting exactly the two orphans (at the library level) clears the tree.
    assert!(orphans_with(
        &fixture("python_exempt"),
        Language::Python,
        &exempt(&["cli.py", "pkg/__init__.py"]),
    )
    .is_empty());
}

#[test]
fn python_subcommand_exits_zero_with_config_exemptions() {
    // The fixture's testing-conventions.toml exempts cli.py + pkg/__init__.py.
    assert_eq!(unit_colocated_test_exit("python_exempt", "python"), 0);
}

#[test]
fn a_typescript_barrel_is_an_orphan_until_explicitly_exempted() {
    // No automatic shape exemption: a re-export barrel is an orphan…
    assert_eq!(
        relative_orphans(&fixture("typescript/exempt"), Language::TypeScript),
        vec!["index.ts"],
    );
    // …until the config exempts it, with a reason.
    assert_eq!(
        unit_colocated_test_exit("typescript/exempt", "typescript"),
        0
    );
}

#[test]
fn a_stale_exempt_entry_is_an_error() {
    // The fixture exempts `ghost.py`, which doesn't exist — config can't rot.
    assert!(
        unit_colocated_test_run("stale_exempt", "python").is_err(),
        "an exempt entry pointing at a missing file must error"
    );
}

// ---- CLI surface (#22) ---------------------------------------------------

/// Raw result of invoking the CLI with `args` after the program name, so a
/// usage error (clap) can be asserted rather than unwrapped away.
fn run_cli(args: &[&str]) -> anyhow::Result<i32> {
    let argv: Vec<OsString> = std::iter::once(OsString::from("testing-conventions"))
        .chain(args.iter().copied().map(OsString::from))
        .collect();
    run(argv)
}

#[test]
fn unit_colocated_test_requires_language() {
    // Omitting `--language` is a usage error — never a silent `python` run.
    let err = run_cli(&["unit", "colocated-test", "src"]).expect_err("--language is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn the_flat_unit_location_subcommand_is_gone() {
    // The pre-#22 flat form (`unit-location …`) no longer parses; the rule now
    // lives at `unit colocated-test`.
    let err = run_cli(&["unit-location", "src"]).expect_err("the flat subcommand was removed");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("an unknown subcommand should surface as a clap::Error");
    assert_eq!(clap_err.kind(), clap::error::ErrorKind::InvalidSubcommand);
}

#[test]
fn the_old_unit_location_subcommand_is_renamed() {
    // #55: the rule was renamed `location` → `colocated-test`, so the former
    // `unit location` no longer parses; only `unit colocated-test` does.
    let err = run_cli(&["unit", "location", "src"])
        .expect_err("`unit location` was renamed to `unit colocated-test`");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("an unknown subcommand should surface as a clap::Error");
    assert_eq!(clap_err.kind(), clap::error::ErrorKind::InvalidSubcommand);
}
