//! Clean: the temp directory's name carries the runner's own PID so concurrent tests do not
//! collide. `std::process::id` is the other half of the `env::temp_dir` carve-out — it reports
//! the runner, not ambient state, and there is nothing to inject a double for.

use std::path::Path;

fn stem(path: &Path) -> String {
    path.file_stem()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_stem_of_a_process_unique_name() {
        let dir = std::env::temp_dir().join(format!("isolation-clean-{}", std::process::id()));

        assert_eq!(
            stem(&dir),
            format!("isolation-clean-{}", std::process::id())
        );
    }
}
