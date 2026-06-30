//! Integration tests for the TypeScript mutation rule (#202).
//!
//! These run REAL Stryker over the fixture projects via the SDK
//! ([`mutation::measure_typescript`]) — which spawns the bundled Node adapter (#246) — and
//! assert the surviving-mutant set, the TS parallel of the Rust vertical (#201). Per the #3
//! guardrail the *projects themselves* are the fixtures: `killed` (every mutant caught by an
//! asserting test) reports no survivors, and `survivors` (an assertion-light test that runs
//! the code but pins nothing) reports several — the gap mutation testing exposes that
//! coverage can't.
//!
//! The fixtures are **runner-only**: they install just vitest, never Stryker. That the gate
//! still runs Stryker over them is the proof of #246 — the consumer installs nothing
//! Stryker-related; the tool bundles and drives it. Each test runs against its own staged
//! copy (vitest `node_modules` symlinked) so the parallel Stryker sandboxes never collide,
//! and points the rule at the freshly-built adapter via [`common::ensure_ts_adapter_env`].
//! Requires the built node adapter and the fixtures' vitest (`npm ci` in
//! `tests/fixtures/unit_mutation/typescript`).

mod common;

use common::{ensure_ts_adapter_env, Staged};
use testing_conventions::mutation::measure_typescript;

#[test]
fn killed_reports_no_survivors() {
    ensure_ts_adapter_env();
    let project = Staged::new("killed");
    let survivors = measure_typescript(
        project.path(),
        &[],
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("stryker runs");
    assert!(
        survivors.is_empty(),
        "every mutant should be caught; got {survivors:?}"
    );
}

#[test]
fn survivors_are_reported() {
    // The fixture installs only vitest — no Stryker — yet the gate runs Stryker over it via
    // the bundled adapter and finds the assertion-light suite's survivors. That's #246: the
    // consumer installs nothing Stryker-related.
    ensure_ts_adapter_env();
    let project = Staged::new("survivors");
    let survivors = measure_typescript(
        project.path(),
        &[],
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("stryker runs");
    assert!(
        !survivors.is_empty(),
        "the assertion-light suite should leave survivors"
    );
    assert!(
        survivors.iter().all(|m| m.file == "index.ts"),
        "every survivor is in index.ts; got {survivors:?}"
    );
}

#[test]
fn a_mutation_exemption_drops_the_survivors() {
    // Exempting the survivors' file lifts all of them — an equivalent / deliberately
    // defensive mutation, waived with a reason via `[[typescript.exempt]] rules = ["mutation"]`.
    ensure_ts_adapter_env();
    let project = Staged::new("survivors");
    let exempt = vec!["index.ts".to_string()];
    let survivors = measure_typescript(
        project.path(),
        &exempt,
        &std::collections::BTreeMap::new(),
        None,
    )
    .expect("stryker runs");
    assert!(
        survivors.is_empty(),
        "the exemption should drop every survivor; got {survivors:?}"
    );
}
