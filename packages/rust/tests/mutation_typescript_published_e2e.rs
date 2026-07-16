//! E2E test for the TypeScript mutation rule under the **published-package topology**:
//! drive the built CLI binary end to end with the adapter resolved from an isolated
//! install of the packed npm package ([`common::PublishedInstall`]) — the dependency tree
//! `npx -y testing-conventions` runs in production, where a devDependency of this repo
//! (`typescript` among them) is absent from every resolution path. The fixture is the
//! package-shaped `upward_survivors` project carrying a `tsconfig.json`; the run must be
//! judged on its mutants — the survivors fail the gate — never on Stryker's own module
//! resolution at startup.
//!
//! Requires the built node package (`pnpm run build` in `packages/node`), the fixtures'
//! vitest (`npm ci` in `tests/fixtures/unit_mutation/typescript`), and registry access
//! for the isolated install.

mod common;

use std::process::Command;

use common::{PublishedInstall, Staged};

#[test]
fn a_tsconfig_package_fails_on_its_survivors_through_the_published_adapter() {
    let install = PublishedInstall::new();
    let package = Staged::upward("upward_survivors");
    let out = Command::new(env!("CARGO_BIN_EXE_testing-conventions"))
        .args(["unit", "mutation", "--language", "typescript"])
        .arg("--ts-mutation-adapter")
        .arg(install.adapter())
        .arg(package.path().join("src"))
        .output()
        .expect("the built binary should run");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(1),
        "the assertion-light suite leaves survivors; stderr: {stderr}"
    );
    assert!(
        stderr.contains("unexplained surviving mutant") && stderr.contains("index.ts"),
        "the run is judged on mutants, listed scan-path-relative — not on a startup \
         resolution error; got: {stderr}"
    );
}
