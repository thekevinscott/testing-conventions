mod common;

use std::collections::BTreeMap;
use std::path::PathBuf;

use common::expect_tested;
use testing_conventions::mutation::measure_rust;

fn member() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/unit_mutation/rust/gated_ws/member")
}

#[test]
fn a_feature_gated_integration_test_target_builds_but_its_kills_dont_count() {
    let (count, survivors) = expect_tested(
        measure_rust(
            &member(),
            &[],
            &BTreeMap::new(),
            None,
            &["boost".to_string()],
        )
        .expect("cargo-mutants runs"),
    );
    assert!(
        count > 0,
        "the feature reaches the build phase, so the pass carries its evidence; got {count}"
    );
    assert!(
        !survivors.is_empty(),
        "the gated module's only test lives in the integration tier, which mutation doesn't judge; got {survivors:?}"
    );
}

#[test]
fn a_baseline_that_cannot_build_is_an_error_not_a_vacuous_pass() {
    let err = measure_rust(&member(), &[], &BTreeMap::new(), None, &[])
        .expect_err("a baseline that cannot build fails the measurement");
    assert!(
        err.to_string().contains("did not run cleanly"),
        "the failure names the baseline build; got: {err}"
    );
}
