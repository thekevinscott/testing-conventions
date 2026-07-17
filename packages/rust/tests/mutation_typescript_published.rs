//! Integration test for the TypeScript mutation rule under the **published-package
//! topology**: the adapter is resolved from an isolated install of the packed npm package
//! ([`common::PublishedInstall`]) — the dependency tree `npx -y testing-conventions` runs
//! in production — instead of from this repo's dev tree, where hoisted devDependencies
//! (`typescript` among them) sit on Stryker's resolution path and mask a
//! missing-declared-dependency bug. The fixture is the package-shaped `upward_survivors`
//! project carrying a `tsconfig.json` — the file that activates Stryker's ts-config
//! machinery, whose sandbox-copy path imports `typescript` from `@stryker-mutator/core`'s
//! own location. The gate must reach a mutant verdict, not die at startup.
//!
//! Requires the built node package (`pnpm run build` in `packages/node`), the fixtures'
//! vitest (`npm ci` in `tests/fixtures/unit_mutation/typescript`), and registry access
//! for the isolated install.

mod common;

use common::{expect_tested, PublishedInstall, Staged};
use testing_conventions::mutation::measure_typescript;

#[test]
fn a_tsconfig_package_reaches_a_mutant_verdict_through_the_published_adapter() {
    let install = PublishedInstall::new();
    let package = Staged::upward("upward_survivors");
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
