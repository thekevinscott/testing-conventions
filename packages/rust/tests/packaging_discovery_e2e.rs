use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

fn fixture(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/packaging")
        .join(relative)
}

/// A scratch root holding copies of named fixture distributions, removed on drop.
struct DistRoot(PathBuf);

impl DistRoot {
    fn new(entries: &[(&str, &str)]) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-packaging-discovery-e2e-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        for (source, destination) in entries {
            let target = root.join(destination);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(fixture(source), &target).unwrap();
        }
        DistRoot(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for DistRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Exit code, stdout and stderr of `testing-conventions packaging <path>`.
fn packaging(path: &Path) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .arg("packaging")
        .arg(path)
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
fn a_root_of_clean_distributions_exits_zero() {
    let root = DistRoot::new(&[
        ("python_wheel/clean.whl", "clean.whl"),
        ("python_sdist/clean-0.1.0.tar.gz", "clean-0.1.0.tar.gz"),
        ("typescript_npm/clean.tgz", "clean.tgz"),
        ("rust_crate/clean-0.1.0.crate", "clean-0.1.0.crate"),
    ]);

    let (code, stdout, _) = packaging(root.path());

    assert_eq!(code, 0, "{stdout}");
    assert!(
        stdout.contains("checked 4 built distribution(s)"),
        "the run reports what it covered: {stdout}"
    );
}

#[test]
fn a_root_holding_a_distribution_that_ships_a_test_file_exits_nonzero() {
    let root = DistRoot::new(&[
        ("python_wheel/clean.whl", "clean.whl"),
        ("typescript_npm/red.tgz", "red.tgz"),
    ]);

    let (code, _, stderr) = packaging(root.path());

    assert_eq!(code, 1, "{stderr}");
    assert!(
        stderr.contains("red.tgz"),
        "the failure names the distribution that shipped the test file: {stderr}"
    );
}

#[test]
fn a_root_holding_no_recognized_distribution_exits_nonzero() {
    let root = DistRoot::new(&[]);

    let (code, _, stderr) = packaging(root.path());

    assert_eq!(code, 1, "{stderr}");
    assert!(
        stderr.contains("no recognized built distribution"),
        "the failure names the constraint: {stderr}"
    );
}

#[test]
fn a_named_distribution_needs_no_language() {
    let (code, _, stderr) = packaging(&fixture("rust_crate/widget-0.1.0.crate"));

    assert_eq!(code, 1, "{stderr}");
}
