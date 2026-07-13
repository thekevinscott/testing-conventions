//! Integration tests for the unit `colocated-test` check.
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

use testing_conventions::colocated_test::{missing_inline_tests, missing_unit_tests, Language};
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

#[test]
fn rust_clean_tree_exits_zero() {
    // Every source module with testable behavior carries an inline `#[cfg(test)]`
    // module; module-declaration (`lib.rs`) and type-only files are not subjects.
    assert_eq!(unit_colocated_test_exit("rust/clean", "rust"), 0);
}

#[test]
fn rust_red_tree_exits_nonzero() {
    // `src/untested.rs` has a function but no inline `#[cfg(test)]` module, so the
    // presence check must flag it; the correctly-tested `widget.rs` must not be.
    assert_eq!(unit_colocated_test_exit("rust/red", "rust"), 1);
}

#[test]
fn rust_cfg_not_test_module_is_not_a_test_module() {
    // `#[cfg(not(test))]` compiles in non-test builds — it is production code,
    // never a test module. A file with behavior whose only cfg-gated module is
    // `not(test)` has no inline test, so it is an orphan the presence gate must
    // flag — not a false green.
    let root = fixture("rust/cfg_not_test");
    let orphans: Vec<String> = missing_inline_tests(&root, &BTreeSet::new())
        .expect("walking a readable tree should succeed")
        .iter()
        .map(|path| {
            path.strip_prefix(&root)
                .expect("an orphan must live under the scanned root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    assert_eq!(orphans, vec!["src/gated.rs"]);
}

#[test]
fn rust_cfg_not_test_tree_exits_nonzero() {
    // The subcommand must surface the orphan as a non-zero exit.
    assert_eq!(unit_colocated_test_exit("rust/cfg_not_test", "rust"), 1);
}

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
fn a_type_only_typescript_module_is_not_a_subject() {
    // A `.ts` module of only type/interface/`import type`/`export type` declarations
    // compiles to zero runtime JS — no behavior to unit-test — so it is not a
    // colocated-test subject, exactly like a `.d.ts` declaration file. The fixture's
    // `shape.ts` / `aliases.ts` are type-only and untested; `widget.ts` is paired.
    assert!(
        relative_orphans(&fixture("typescript/type_only"), Language::TypeScript).is_empty(),
        "type-only modules are not subjects and need no colocated test or exemption"
    );
    assert_eq!(
        unit_colocated_test_exit("typescript/type_only", "typescript"),
        0
    );
}

#[test]
fn a_typescript_module_mixing_a_type_and_runtime_code_stays_a_subject() {
    // The boundary: `mixed.ts` pairs a `type` with a runtime `const`, so it has
    // behavior and remains an orphan until it gets a colocated test. Guards the
    // type-only recognition against over-skipping a module that ships runtime code.
    assert_eq!(
        relative_orphans(&fixture("typescript/type_only_mixed"), Language::TypeScript),
        vec!["mixed.ts"],
    );
    assert_eq!(
        unit_colocated_test_exit("typescript/type_only_mixed", "typescript"),
        1
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
    // The flat form (`unit-location …`) no longer parses; the rule now
    // lives at `unit colocated-test`.
    let err = run_cli(&["unit-location", "src"]).expect_err("the flat subcommand was removed");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("an unknown subcommand should surface as a clap::Error");
    assert_eq!(clap_err.kind(), clap::error::ErrorKind::InvalidSubcommand);
}

#[test]
fn the_old_unit_location_subcommand_is_renamed() {
    // The rule was renamed `location` → `colocated-test`, so the former
    // `unit location` no longer parses; only `unit colocated-test` does.
    let err = run_cli(&["unit", "location", "src"])
        .expect_err("`unit location` was renamed to `unit colocated-test`");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("an unknown subcommand should surface as a clap::Error");
    assert_eq!(clap_err.kind(), clap::error::ErrorKind::InvalidSubcommand);
}

#[test]
fn the_unit_co_change_subcommand_is_folded_into_base() {
    // `unit co-change` was folded into `unit colocated-test --base`, so the
    // standalone subcommand no longer parses.
    let err = run_cli(&[
        "unit",
        "co-change",
        "src",
        "--language",
        "python",
        "--base",
        "HEAD",
    ])
    .expect_err("`unit co-change` was folded into `unit colocated-test --base`");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("an unknown subcommand should surface as a clap::Error");
    assert_eq!(clap_err.kind(), clap::error::ErrorKind::InvalidSubcommand);
}

#[test]
fn python_suite_helpers_are_not_colocated_subjects() {
    // `tests/integration/helper.py` and `tests/e2e/support.py` are suite
    // support under `<package root>/tests/`, which belongs to the suite tiers —
    // never subjects of the colocated-unit rule.
    assert_eq!(unit_colocated_test_exit("python_tiers", "python"), 0);
}

#[test]
fn typescript_suite_helpers_are_not_colocated_subjects() {
    // `tests/integration/helper.ts` is suite support under
    // `<package root>/tests/` — never a subject of the colocated-unit rule.
    assert_eq!(
        unit_colocated_test_exit("typescript_tiers", "typescript"),
        0
    );
}
