use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::run;

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
            "tc-packaging-discovery-{}-{}",
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

/// `testing-conventions packaging <path>`, run in process.
fn packaging(path: &Path) -> anyhow::Result<i32> {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "packaging".into(),
        path.to_path_buf().into_os_string(),
    ];
    run(argv)
}

#[test]
fn a_root_of_clean_distributions_passes_without_a_language() {
    let root = DistRoot::new(&[
        ("python_wheel/clean.whl", "clean.whl"),
        ("python_sdist/clean-0.1.0.tar.gz", "clean-0.1.0.tar.gz"),
        ("typescript_npm/clean.tgz", "clean.tgz"),
        ("rust_crate/clean-0.1.0.crate", "clean-0.1.0.crate"),
    ]);

    assert_eq!(packaging(root.path()).unwrap(), 0);
}

#[test]
fn each_distribution_is_checked_under_the_language_its_extension_names() {
    for entry in [
        ("python_wheel/red.whl", "red.whl"),
        ("python_sdist/widget-0.1.0.tar.gz", "widget-0.1.0.tar.gz"),
        ("typescript_npm/red.tgz", "red.tgz"),
        ("rust_crate/widget-0.1.0.crate", "widget-0.1.0.crate"),
    ] {
        let root = DistRoot::new(&[entry]);

        assert_eq!(packaging(root.path()).unwrap(), 1, "{} passed", entry.0);
    }
}

#[test]
fn one_red_distribution_fails_a_root_of_otherwise_clean_ones() {
    let root = DistRoot::new(&[
        ("python_wheel/clean.whl", "clean.whl"),
        ("typescript_npm/red.tgz", "red.tgz"),
        ("rust_crate/clean-0.1.0.crate", "clean-0.1.0.crate"),
    ]);

    assert_eq!(packaging(root.path()).unwrap(), 1);
}

#[test]
fn discovery_reaches_a_distribution_in_a_subdirectory() {
    let root = DistRoot::new(&[("python_wheel/red.whl", "nested/deeper/red.whl")]);

    assert_eq!(packaging(root.path()).unwrap(), 1);
}

#[test]
fn a_root_holding_no_recognized_distribution_is_an_error() {
    let root = DistRoot::new(&[]);

    let err = packaging(root.path()).expect_err("an empty root must fail the run");

    assert!(
        format!("{err:#}").contains("no recognized built distribution"),
        "got: {err:#}"
    );
}

#[test]
fn a_file_that_is_no_distribution_is_not_discovered() {
    let root = DistRoot::new(&[("python_clean/widget.py", "widget.py")]);

    let err = packaging(root.path()).expect_err("a root of plain files must fail the run");

    assert!(
        format!("{err:#}").contains("no recognized built distribution"),
        "got: {err:#}"
    );
}

#[test]
fn a_named_distribution_infers_its_language_from_its_extension() {
    assert_eq!(packaging(&fixture("typescript_npm/red.tgz")).unwrap(), 1);
    assert_eq!(packaging(&fixture("typescript_npm/clean.tgz")).unwrap(), 0);
}

#[test]
fn a_named_language_still_checks_a_named_distribution() {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "packaging".into(),
        fixture("python_wheel/red.whl").into_os_string(),
        "--language".into(),
        "python".into(),
    ];

    assert_eq!(run(argv).unwrap(), 1);
}

#[test]
fn a_named_language_still_checks_an_unpacked_artifact_root() {
    let argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "packaging".into(),
        fixture("python_red").into_os_string(),
        "--language".into(),
        "python".into(),
    ];

    assert_eq!(run(argv).unwrap(), 1);
}
