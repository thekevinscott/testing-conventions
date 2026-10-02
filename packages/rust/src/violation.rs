//! The shared `Violation` type emitted by the deterministic test-code lints.

use std::path::PathBuf;

/// A single lint violation found in a test file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    pub file: PathBuf,
    /// 1-based line number of the offending construct.
    pub line: usize,
    /// Short lint identifier (e.g. `no-monkeypatch`, `no-out-of-module-call`).
    pub rule: &'static str,
    pub message: String,
}

/// A violation rendered for a terminal: `file:line: rule — message`.
pub fn rendered(violation: &Violation) -> String {
    format!(
        "{}:{}: {} — {}",
        violation.file.display(),
        violation.line,
        violation.rule,
        violation.message
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_violation_renders_its_file_line_rule_and_message() {
        let rendering = rendered(&Violation {
            file: PathBuf::from("src/widget.rs"),
            line: 12,
            rule: "no-monkeypatch",
            message: "patches `os.environ`".to_string(),
        });

        assert_eq!(
            rendering,
            "src/widget.rs:12: no-monkeypatch — patches `os.environ`"
        );
    }
}
