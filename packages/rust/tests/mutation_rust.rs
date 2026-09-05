mod common;

use std::path::PathBuf;

use common::expect_tested;
use testing_conventions::mutation::measure_rust;

fn crate_dir(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_mutation/rust")
        .join(name)
}

#[test]
fn killed_reports_no_survivors_and_counts_the_tested_mutants() {
    let (count, survivors) = expect_tested(
        measure_rust(
            &crate_dir("killed"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
    assert!(count > 0, "the engine judged the crate's mutants");
}

#[test]
fn survivors_are_reported() {
    let (count, survivors) = expect_tested(
        measure_rust(
            &crate_dir("survivors"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors"
    );
    assert!(
        survivors.iter().all(|m| m.file == "src/lib.rs"),
        "every survivor is in src/lib.rs; got {survivors:?}"
    );
    assert!(
        count >= survivors.len(),
        "every survivor was judged, so the count covers them"
    );
    assert!(
        survivors.iter().all(|m| m.description.contains(" with ")),
        "each survivor names the source its mutation produced; got {survivors:?}"
    );
}

#[test]
fn survivor_descriptions_carry_no_location_prefix() {
    let (_, survivors) = expect_tested(
        measure_rust(
            &crate_dir("survivors"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors"
    );
    assert!(
        survivors.iter().all(|m| !m.description.contains(&m.file)),
        "a survivor's description carries no embedded location; got {survivors:?}"
    );
}

#[test]
fn a_crate_with_no_mutants_reports_a_zero_count() {
    let (count, survivors) = expect_tested(
        measure_rust(
            &crate_dir("no_mutants"),
            &[],
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert_eq!(count, 0, "constants offer nothing to mutate");
    assert!(survivors.is_empty(), "got {survivors:?}");
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    let exempt = vec!["src/lib.rs".to_string()];
    let (_, survivors) = expect_tested(
        measure_rust(
            &crate_dir("survivors"),
            &exempt,
            &std::collections::BTreeMap::new(),
            None,
            &[],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}

#[test]
fn a_scan_path_outside_any_workspace_is_an_error() {
    let dir = std::env::temp_dir().join(format!("tc-mut-rust-nows-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let err = measure_rust(&dir, &[], &std::collections::BTreeMap::new(), None, &[]).unwrap_err();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        format!("{err:#}").contains("cargo locate-project failed"),
        "got: {err:#}"
    );
}

#[test]
fn a_bad_base_ref_is_an_error() {
    let dir = std::env::temp_dir().join(format!("tc-mut-rust-badref-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"probe\"\nversion = \"0.0.1\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn probe() {}\n").unwrap();
    let init = std::process::Command::new("git")
        .current_dir(&dir)
        .args(["init", "-q"])
        .status()
        .unwrap();
    assert!(init.success());
    let err = measure_rust(
        &dir,
        &[],
        &std::collections::BTreeMap::new(),
        Some("tc-no-such-ref"),
        &[],
    )
    .unwrap_err();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        format!("{err:#}").contains("git diff tc-no-such-ref...HEAD failed"),
        "got: {err:#}"
    );
}
