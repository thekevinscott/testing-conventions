use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::isolation::find_violations;
use testing_conventions::run;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/isolation")
        .join(name)
}

/// Exit code of `unit lint --language rust <fixture>`. `--config` names the fixture's own
/// file — usually absent, meaning nothing is exempt — so that `--config`'s CWD-relative
/// default cannot pull this crate's config in over a fixture tree.
fn iso_exit(fixture_name: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        "--config".into(),
        fixture(fixture_name)
            .join("testing-conventions.toml")
            .into_os_string(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

/// `true` when scanning `fixture_name` yields an `no-out-of-module-call` violation
/// in the file ending `file_suffix`.
fn flagged(fixture_name: &str, file_suffix: &str) -> bool {
    find_violations(fixture(fixture_name), fixture(fixture_name))
        .expect("walking a readable tree should succeed")
        .iter()
        .any(|v| v.rule == "no-out-of-module-call" && v.file.ends_with(file_suffix))
}

#[test]
fn red_flags_first_party_cross_module_call() {
    assert!(
        flagged("unit/red", "cross_module.rs"),
        "`crate::store::load()` in a unit test must be flagged"
    );
}

#[test]
fn red_flags_effectful_std_call() {
    assert!(
        flagged("unit/red", "effectful_std.rs"),
        "`std::net::TcpStream::connect(...)` in a unit test must be flagged"
    );
}

#[test]
fn red_flags_a_spawned_process() {
    assert!(
        flagged("unit/red", "spawned_process.rs"),
        "`std::process::Command::new(...)` in a unit test must be flagged"
    );
}

#[test]
fn clean_allows_a_process_unique_temp_name() {
    assert!(
        !flagged("unit/clean", "unique_temp_name.rs"),
        "`std::process::id` reports the runner's own PID, so it names a temp directory the \
         way `env::temp_dir` does rather than reading ambient state"
    );
}

#[test]
fn clean_allows_a_fixture_read() {
    assert!(
        !flagged("unit/clean", "fixture_read.rs"),
        "`std::fs` and `std::env::temp_dir` build the tree a private walker needs, and the \
         inline module is the only tier that can reach a private item"
    );
}

#[test]
fn red_flags_external_crate_call() {
    assert!(
        flagged("unit/red", "external_crate.rs"),
        "`rand::random()` in a unit test must be flagged"
    );
}

#[test]
fn red_flags_ancestor_module_reach() {
    assert!(
        flagged("unit/red", "ancestor.rs"),
        "`super::super::util::help()` in a unit test must be flagged"
    );
}

#[test]
fn clean_reports_no_violations() {
    let violations = find_violations(fixture("unit/clean"), fixture("unit/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture is well-isolated (super:: + injected double + Cursor); got {violations:?}"
    );
}

#[test]
fn red_exits_nonzero() {
    assert_eq!(iso_exit("unit/red"), 1);
}

#[test]
fn clean_exits_zero() {
    assert_eq!(iso_exit("unit/clean"), 0);
}

#[test]
fn cfg_not_test_module_is_not_linted_as_test_code() {
    let violations = find_violations(fixture("unit/cfg_not_test"), fixture("unit/cfg_not_test"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "production code under `#[cfg(not(test))]` must not be linted as a unit test; got {violations:?}"
    );
}

#[test]
fn cfg_not_test_exits_zero() {
    assert_eq!(iso_exit("unit/cfg_not_test"), 0);
}

/// `true` when scanning `fixture_name` yields a `no-out-of-module-import`
/// violation in the file ending `file_suffix`.
fn import_flagged(fixture_name: &str, file_suffix: &str) -> bool {
    find_violations(fixture(fixture_name), fixture(fixture_name))
        .expect("walking a readable tree should succeed")
        .iter()
        .any(|v| v.rule == "no-out-of-module-import" && v.file.ends_with(file_suffix))
}

#[test]
fn imports_red_flags_first_party_glob() {
    assert!(
        import_flagged("imports/red", "first_party_glob.rs"),
        "`use crate::other::*` in a unit test must be flagged"
    );
}

#[test]
fn imports_red_flags_first_party_named() {
    assert!(
        import_flagged("imports/red", "first_party_named.rs"),
        "`use crate::other::Thing` in a unit test must be flagged"
    );
}

#[test]
fn imports_red_flags_external_crate() {
    assert!(
        import_flagged("imports/red", "external_named.rs"),
        "`use rand::Rng` in a unit test must be flagged"
    );
}

#[test]
fn imports_red_flags_effectful_std() {
    assert!(
        import_flagged("imports/red", "effectful_std.rs"),
        "`use std::net` in a unit test must be flagged"
    );
}

#[test]
fn imports_clean_allows_the_filesystem() {
    assert!(
        !import_flagged("unit/clean", "fixture_read.rs"),
        "`use std::fs` follows the call: a unit test may build the tree it walks"
    );
}

#[test]
fn imports_clean_reports_no_violations() {
    let violations = find_violations(fixture("imports/clean"), fixture("imports/clean"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations.is_empty(),
        "the clean fixture imports only super:: and pure std; got {violations:?}"
    );
}

#[test]
fn imports_red_exits_nonzero() {
    assert_eq!(iso_exit("imports/red"), 1);
}

#[test]
fn imports_clean_exits_zero() {
    assert_eq!(iso_exit("imports/clean"), 0);
}

#[test]
fn isolation_requires_language() {
    let err =
        run(["testing-conventions", "unit", "lint", "src"]).expect_err("--language is required");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("a missing required flag should surface as a clap::Error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::MissingRequiredArgument
    );
}

/// Exit code of `unit lint --language rust --config <config> <fixture>`.
fn iso_exit_config(fixture_name: &str, config_rel: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        "--config".into(),
        fixture(config_rel).into_os_string(),
        fixture(fixture_name).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn waived_out_of_module_call_exits_zero() {
    assert_eq!(
        iso_exit_config("unit/waived", "unit/waived/testing-conventions.toml"),
        0
    );
}

#[test]
fn stale_exempt_entry_errors() {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        "--config".into(),
        fixture("unit/stale_exempt.toml").into_os_string(),
        fixture("unit/waived").into_os_string(),
    ];
    assert!(
        run(argv).is_err(),
        "a stale exempt entry must error, not silently pass"
    );
}

#[test]
fn local_build_crate_neither_aborts_nor_false_flags() {
    let violations = find_violations(fixture("unit/local_build"), fixture("unit/local_build"))
        .expect("a locally-built crate must not abort the rule on a tests/ or target/ file");
    assert!(
        violations.is_empty(),
        "target/ and tests/ must be skipped, so nothing is flagged; got {violations:?}"
    );
}

#[test]
fn local_build_crate_exits_zero() {
    assert_eq!(iso_exit("unit/local_build"), 0);
}

/// Every violation line in `fixture_name`'s file ending `file_suffix`.
fn lines_in(fixture_name: &str, file_suffix: &str) -> Vec<usize> {
    find_violations(fixture(fixture_name), fixture(fixture_name))
        .expect("walking a readable tree should succeed")
        .iter()
        .filter(|v| v.file.ends_with(file_suffix))
        .map(|v| v.line)
        .collect()
}

#[test]
fn a_reach_inside_assert_is_caught() {
    let lines = lines_in("unit/macro_body", "asserted.rs");
    assert!(
        lines.contains(&14),
        "`assert!(crate::other::load())` hides a first-party reach in a token body; got {lines:?}"
    );
}

/// Exit code of `unit lint --language rust <fixture>/<sub>`, naming the fixture's own config.
fn iso_exit_subdir(fixture_name: &str, sub: &str) -> i32 {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "lint".into(),
        "--language".into(),
        "rust".into(),
        "--config".into(),
        fixture(fixture_name)
            .join("testing-conventions.toml")
            .into_os_string(),
        fixture(fixture_name).join(sub).into_os_string(),
    ];
    run(argv).expect("a readable tree should not error")
}

#[test]
fn scanning_src_still_reads_the_crate_manifest() {
    let violations = find_violations(fixture("unit/red").join("src"), fixture("unit/red"))
        .expect("walking a readable tree should succeed");
    assert!(
        violations
            .iter()
            .any(|v| v.file.ends_with("external_crate.rs")),
        "`rand::random()` must stay flagged when the scan path is `src/`; got {violations:?}"
    );
}

#[test]
fn a_direct_and_an_asserted_reach_agree() {
    let lines = lines_in("unit/macro_body", "asserted.rs");
    assert!(
        lines.contains(&9) && lines.contains(&14),
        "the direct and macro-wrapped forms of the same reach must both be flagged; got {lines:?}"
    );
}

#[test]
fn scanning_src_exits_nonzero() {
    assert_eq!(
        iso_exit_subdir("unit/red", "src"),
        1,
        "pointing the gate one level down must not turn a red tree green"
    );
}

#[test]
fn an_external_reach_among_assert_eq_args_is_caught() {
    let lines = lines_in("unit/macro_body", "asserted.rs");
    assert!(
        lines.contains(&19),
        "every comma-separated argument is a written expression, including the format \
         arguments; got {lines:?}"
    );
}

#[test]
fn a_quoted_template_is_not_a_reach() {
    let lines = lines_in("unit/macro_body", "quoted.rs");
    assert!(
        lines.is_empty(),
        "`quote!` tokens are a template for code emitted elsewhere, not a call made here; \
         got {lines:?}"
    );
}

#[test]
fn an_import_inside_an_item_body_macro_is_caught() {
    let imports: Vec<_> = find_violations(fixture("unit/macro_body"), fixture("unit/macro_body"))
        .expect("walking a readable tree should succeed")
        .into_iter()
        .filter(|v| v.rule == "no-out-of-module-import" && v.file.ends_with("item_body.rs"))
        .collect();
    assert!(
        !imports.is_empty(),
        "a macro body that parses as items still carries a written `use`; got {imports:?}"
    );
}

#[test]
fn macro_body_exits_nonzero() {
    assert_eq!(iso_exit("unit/macro_body"), 1);
}

#[test]
fn an_exempt_path_stays_crate_root_relative_from_src() {
    assert_eq!(
        iso_exit_subdir("unit/waived", "src"),
        0,
        "`path = \"src/widget.rs\"` must resolve from either scan path — the exempt root \
         is the crate root, not the scan path"
    );
}

#[test]
fn this_crate_has_no_out_of_module_command_reaches() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let reaches: Vec<_> = find_violations(&root, &root)
        .unwrap()
        .into_iter()
        .filter(|v| v.message.contains("crate::command"))
        .collect();
    assert!(
        reaches.is_empty(),
        "tests/workflow.rs covers the real command tree, so no inline test needs to \
         build one; got {reaches:?}"
    );
}
