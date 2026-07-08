//! Integration tests for line-scoped exemptions in the config schema.
//!
//! Today an exemption is file-scoped: `[[<lang>.exempt]] rules = ["coverage"]` lifts a
//! rule for an *entire* file. An optional `lines` key narrows a
//! `coverage` / `mutation` exemption to the exact lines it covers, so a single stubborn
//! line (an equivalent mutant, a cross-version import shim) no longer forces the whole
//! module past the gate. This pins the loader half of that contract through real
//! fixture files: a `lines` list parses, and the schema's self-guard rejects the misuse.

use std::path::PathBuf;

use testing_conventions::config::{load_config, LineSpec, Rule};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn loads_a_line_scoped_exemption() {
    // `lines = [9, 10, "12-13"]` on a coverage/mutation exemption parses into single
    // lines and an inclusive range — the motivating tomlcompat.py case.
    let config = load_config(fixture("exempt_lines.toml")).expect("a line-scoped exemption loads");
    let exempt = &config.python.expect("[python] table").exempt[0];
    assert_eq!(exempt.path, "src/tomlcompat.py");
    assert_eq!(exempt.rules, vec![Rule::Coverage, Rule::Mutation]);
    assert_eq!(
        exempt.lines,
        vec![
            LineSpec::Single(9),
            LineSpec::Single(10),
            LineSpec::Range(12, 13),
        ]
    );
    assert_eq!(
        exempt.line_set().into_iter().collect::<Vec<_>>(),
        vec![9, 10, 12, 13]
    );
}

#[test]
fn rejects_lines_on_a_whole_file_rule_self_guard() {
    // `colocated-test` is whole-file presence, so a `lines` key alongside it can't mean
    // anything — the loader's self-guard rejects it.
    assert!(
        load_config(fixture("exempt_lines_bad_rule.toml")).is_err(),
        "a `lines` list on `colocated-test` must be rejected on load"
    );
}
