//! Integration tests for line-scoped exemptions in the config schema (#226).
//!
//! Today an exemption is file-scoped: `[[<lang>.exempt]] rules = ["coverage"]`
//! lifts a rule for an *entire* file. #226 adds an optional `lines` key that narrows
//! a `coverage` / `mutation` exemption to the exact lines it covers, so a single
//! stubborn line (an equivalent mutant, a cross-version import shim) no longer forces
//! the whole module past the gate. This pins the loader half of that contract: a
//! `lines` list parses, and the schema's self-guard still rejects the misuse cases.
//!
//! Red until the `lines` key lands: today `serde`'s `deny_unknown_fields` rejects it,
//! so `exempt_lines.toml` fails to load.

use std::path::PathBuf;

use testing_conventions::config::load_config;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn loads_a_line_scoped_exemption() {
    // `lines = [9, 10, "12-13"]` on a coverage/mutation exemption must parse — the
    // motivating tomlcompat.py case from the #218 review.
    let result = load_config(fixture("exempt_lines.toml"));
    assert!(
        result.is_ok(),
        "a line-scoped exemption should load, got: {result:?}"
    );
}
