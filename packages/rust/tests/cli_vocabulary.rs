use std::ffi::OsString;
use std::path::PathBuf;

use testing_conventions::{command, run};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/colocated_test")
        .join(name)
}

/// Long `--help` text for `unit mutation`, rendered in-process from the command tree
/// (`docs/reference/glossary.md`'s "check" is what this subcommand names itself).
fn unit_mutation_help() -> String {
    command()
        .find_subcommand_mut("unit")
        .expect("`unit` is a live subcommand")
        .find_subcommand_mut("mutation")
        .expect("`mutation` is a live subcommand")
        .render_long_help()
        .to_string()
}

/// Long `--help` text for `e2e verify`.
fn e2e_verify_help() -> String {
    command()
        .find_subcommand_mut("e2e")
        .expect("`e2e` is a live subcommand")
        .find_subcommand_mut("verify")
        .expect("`verify` is a live subcommand")
        .render_long_help()
        .to_string()
}

#[test]
fn unit_mutation_help_names_a_check_not_a_gate() {
    let help = unit_mutation_help();
    assert!(
        help.contains("The check is on by default"),
        "got: {help}"
    );
}

#[test]
fn e2e_verify_help_names_checks_not_gates_for_diff_scoping() {
    let help = e2e_verify_help();
    assert!(
        help.contains("the changed-line coverage/mutation checks read the diff"),
        "got: {help}"
    );
}

#[test]
fn mutation_typescript_without_the_adapter_names_the_check_not_the_rule() {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "mutation".into(),
        "--language".into(),
        "typescript".into(),
        fixture("bad_exempt").into_os_string(),
    ];
    let err = run(argv).expect_err("a missing --ts-mutation-adapter must fail the run");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("run the check through that CLI, not the raw binary"),
        "got: {msg}"
    );
}

fn unit_colocated_test_run(fixture_name: &str) -> anyhow::Result<i32> {
    let dir = fixture(fixture_name);
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "colocated-test".into(),
        "--language".into(),
        "python".into(),
        "--config".into(),
        dir.join("testing-conventions.toml").into_os_string(),
        dir.into_os_string(),
    ];
    run(argv)
}

#[test]
fn a_scopable_exemption_without_lines_names_the_check_or_rule_boundary() {
    let err = unit_colocated_test_run("scopable_no_lines")
        .expect_err("a `coverage` exemption with no `lines` must be rejected on load");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("only `coverage` and `mutation` are line-scoped; every other check or rule \
                       is whole-file"),
        "got: {msg}"
    );
}

#[test]
fn lines_alongside_a_whole_file_rule_says_move_the_rest() {
    let err = unit_colocated_test_run("lines_on_whole_file")
        .expect_err("`lines` alongside a whole-file rule must be rejected on load");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("move the rest to a separate entry"),
        "got: {msg}"
    );
}
