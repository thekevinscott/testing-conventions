use std::process::ExitCode;

fn main() -> ExitCode {
    match testing_conventions::run(std::env::args_os()) {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            if let Some(clap_err) = err.downcast_ref::<clap::Error>() {
                clap_err.exit();
            }
            // `{err:#}` prints the whole anyhow chain, so a wrapped failure keeps its context.
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}
