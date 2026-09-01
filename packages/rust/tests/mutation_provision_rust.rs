use std::path::PathBuf;
use std::process::Command;

fn killed_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unit_mutation/rust/killed")
}

/// Exit code of `testing-conventions unit mutation --language rust <crate>`.
fn unit_mutation_exit() -> i32 {
    Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "rust"])
        .arg(killed_crate())
        .status()
        .expect("the built binary should run")
        .code()
        .expect("the process should exit with a code")
}

#[test]
fn provisions_the_engine_and_reuses_it() {
    assert_eq!(
        unit_mutation_exit(),
        0,
        "the tool should provision cargo-mutants and run the gate"
    );
    assert_eq!(
        unit_mutation_exit(),
        0,
        "the provisioned engine should be reused on the next run"
    );
}
