use std::process::Command;

/// Run the built binary with `args`, returning its exit code, stdout, and stderr.
fn cli(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(args)
        .output()
        .expect("the built binary should run");
    (
        out.status
            .code()
            .expect("the process should exit with a code"),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The banner clap's `--version` prints, which shares `CARGO_PKG_VERSION` with it.
fn banner() -> String {
    format!("testing-conventions {}", env!("CARGO_PKG_VERSION"))
}

#[test]
fn a_successful_run_names_its_version() {
    let (code, _, stderr) = cli(&[]);
    assert_eq!(code, 0);
    assert!(
        stderr.contains(&banner()),
        "a passing run names the version that ran: {stderr}"
    );
}

#[test]
fn an_unrecognized_subcommand_still_names_its_version() {
    // The stale-binary signature: an old build rejects a flag a newer workflow
    // passes. The banner precedes parsing so this run still says which build refused.
    let (code, _, stderr) = cli(&["unit", "no-such-rule"]);
    assert_ne!(code, 0);
    assert!(
        stderr.contains(&banner()),
        "a run that dies on parse names the version that refused: {stderr}"
    );
}

#[test]
fn the_banner_stays_off_stdout() {
    // `e2e slug` writes a bare slug a caller reads; the banner must not join it.
    let (_, stdout, _) = cli(&["e2e", "slug", "--branch", "work"]);
    assert!(
        !stdout.contains("testing-conventions "),
        "stdout carries the command's own output alone: {stdout}"
    );
}

#[test]
fn the_banner_matches_the_version_flag() {
    let (_, stdout, _) = cli(&["--version"]);
    assert_eq!(
        stdout.trim(),
        banner(),
        "the banner and `--version` report one version"
    );
}
