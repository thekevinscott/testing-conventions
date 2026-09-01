mod common;

use common::{expect_tested, PublishedInstall, Staged};
use testing_conventions::mutation::measure_typescript;

#[test]
fn a_tsconfig_package_reaches_a_mutant_verdict_through_the_published_adapter() {
    let install = PublishedInstall::new();
    let package = Staged::new("survivors");
    let measurement = measure_typescript(
        &package.path().join("src"),
        &[],
        &std::collections::BTreeMap::new(),
        None,
        &install.adapter(),
    )
    .expect(
        "the run reaches a mutant verdict; startup must not fail on engine-internal resolution",
    );
    let (_, survivors) = expect_tested(measurement);
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors under the scan path"
    );
    assert!(
        survivors.iter().all(|m| m.file == "index.ts"),
        "survivors are reported relative to the scan path; got {survivors:?}"
    );
}
