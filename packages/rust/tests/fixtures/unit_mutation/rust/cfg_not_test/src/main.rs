#[cfg(not(test))]
fn main() -> std::process::ExitCode {
    std::process::ExitCode::from(mut_cfg_not_test::entrypoint::main())
}
