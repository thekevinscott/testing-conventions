//! Integration tests for the CLI's command surface: every subcommand the binary
//! exposes runs a rule and can fail. The `check` umbrella never did — it parsed and
//! returned `Ok(0)` from the same arm as a bare invocation, so it reported a pass no
//! rule had produced.
//!
//! These exercise the library contract — the clap tree (`command()`) and the
//! dispatcher (`run()`) — against the bare invocation that shares `check`'s old arm,
//! so removing one cannot silently take the other with it. The e2e suite
//! (`command_surface_e2e.rs`) drives the same surface through the built binary.

use testing_conventions::{command, run};

#[test]
fn check_is_not_in_the_command_tree() {
    assert!(
        command().find_subcommand("check").is_none(),
        "`check` ran no rule and exited 0; it must not be a subcommand"
    );
}

#[test]
fn check_is_rejected_as_an_unknown_subcommand() {
    let err = run(["testing-conventions", "check"]).expect_err("`check` should not parse");
    let clap_err = err
        .downcast_ref::<clap::Error>()
        .expect("an unknown subcommand is a clap parse error");
    assert_eq!(
        clap_err.kind(),
        clap::error::ErrorKind::InvalidSubcommand,
        "{clap_err}"
    );
}

#[test]
fn no_subcommand_still_exits_zero() {
    // `check` and the no-subcommand path shared one dispatch arm, so this pins the
    // survivor: a bare invocation still parses and returns 0.
    assert_eq!(run(["testing-conventions"]).unwrap(), 0);
}

#[test]
fn the_live_subcommands_are_unchanged() {
    let cli = command();
    let names: Vec<&str> = cli.get_subcommands().map(|c| c.get_name()).collect();
    for name in [
        "install",
        "unit",
        "integration",
        "packaging",
        "workflow",
        "e2e",
    ] {
        assert!(
            names.contains(&name),
            "removing `check` must leave `{name}` in place; got {names:?}"
        );
    }
}
