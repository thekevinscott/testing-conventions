mod common;

use std::process::Command;

use common::{PublishedInstall, Staged};

#[test]
fn a_tsconfig_package_fails_on_its_survivors_through_the_published_adapter() {
    let install = PublishedInstall::new();
    let package = Staged::new("survivors");
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
