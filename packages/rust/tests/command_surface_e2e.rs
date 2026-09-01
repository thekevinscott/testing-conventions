use std::process::Command;

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

#[test]
fn check_exits_nonzero_as_an_unrecognized_subcommand() {
    let (code, _, stderr) = cli(&["check"]);
    assert_ne!(code, 0, "`check` ran no rule, so it must not exit 0");
    assert!(
        stderr.contains("unrecognized subcommand 'check'"),
        "the failure names the subcommand it refused: {stderr}"
    );
}

#[test]
fn help_does_not_list_check() {
    let (_, stdout, _) = cli(&["--help"]);
    assert!(
        !stdout
            .lines()
            .any(|line| line.trim_start().starts_with("check")),
        "`check` must not be listed in --help:\n{stdout}"
    );
}

#[test]
fn no_subcommand_exits_zero_with_the_banner_alone() {
    let (code, stdout, stderr) = cli(&[]);
    assert_eq!(code, 0);
    assert_eq!(stdout, "", "a bare run writes nothing to stdout");
    assert!(
        stderr.contains(&format!(
            "testing-conventions {}",
            env!("CARGO_PKG_VERSION")
        )),
        "a bare run names the version that ran: {stderr}"
    );
}
