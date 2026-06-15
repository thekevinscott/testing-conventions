//! Auditable, reason-required waivers (issue #32).
//!
//! A waiver is an in-file marker that exempts the file it lives in from a named
//! check — the escape hatch for deliberate omissions the tool can't infer (a
//! launcher shim, generated code). It is the opposite of a prose "omit-list"
//! that rots and gets ignored: a waiver lives *at* the omission, shows up in the
//! file's own diff, and **must carry a reason**. A marker with no reason, or an
//! unknown scope, is a hard error — never a silent pass.
//!
//! Grammar, inside any comment (`#` for Python, `//` or `/* … */` for
//! TypeScript — the surrounding comment syntax is irrelevant, the marker is
//! matched as a substring):
//!
//! ```text
//! testing-conventions:waiver(<scope>): <reason>
//! ```
//!
//! `<scope>` is `location`, `coverage`, or `all`; `<reason>` is the rest of the
//! line (a trailing `*/` is trimmed off) and must be non-empty. One waiver per
//! line. The marker token is reserved: a malformed occurrence is an error, so a
//! typo can't quietly disable a check.

use anyhow::{anyhow, bail, Result};

/// The reserved token that introduces a waiver.
const MARKER: &str = "testing-conventions:waiver";

/// The check a waiver can exempt a file from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The unit-test location/naming check ([`crate::location`]).
    Location,
    /// The unit-test coverage floor ([`crate::coverage`]).
    Coverage,
}

/// A parsed waiver: which checks it covers, and the (required) reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waiver {
    location: bool,
    coverage: bool,
    /// Why the omission is deliberate — required, and never empty.
    pub reason: String,
}

impl Waiver {
    /// `true` when this waiver exempts its file from `scope`.
    pub fn covers(&self, scope: Scope) -> bool {
        match scope {
            Scope::Location => self.location,
            Scope::Coverage => self.coverage,
        }
    }
}

/// Parse every waiver marker in `source`.
///
/// A marker that is malformed — missing its `(scope)`, naming an unknown scope,
/// or carrying an empty reason — is an `Err`, so a reasonless waiver can never
/// silently pass. A file with no markers yields an empty `Vec`.
pub fn parse_waivers(source: &str) -> Result<Vec<Waiver>> {
    let mut waivers = Vec::new();
    for line in source.lines() {
        if let Some(start) = line.find(MARKER) {
            waivers.push(parse_marker(&line[start + MARKER.len()..], line)?);
        }
    }
    Ok(waivers)
}

/// The reason `source` is waived for `scope`, if any.
///
/// `Ok(Some(reason))` when a waiver covers `scope`, `Ok(None)` when none does.
/// Any malformed marker in the file — whatever its scope — is an `Err`, so a
/// broken waiver surfaces loudly instead of degrading to "not waived".
pub fn waived_reason(source: &str, scope: Scope) -> Result<Option<String>> {
    Ok(parse_waivers(source)?
        .into_iter()
        .find(|waiver| waiver.covers(scope))
        .map(|waiver| waiver.reason))
}

/// Parse the part of a marker line after [`MARKER`]: `(scope): reason`. `line`
/// is the whole line, for error messages.
fn parse_marker(after_marker: &str, line: &str) -> Result<Waiver> {
    let malformed = || {
        anyhow!(
            "malformed waiver `{}` — expected `{MARKER}(location|coverage|all): <reason>`",
            line.trim()
        )
    };

    let inner = after_marker
        .trim_start()
        .strip_prefix('(')
        .ok_or_else(malformed)?;
    let (scope, rest) = inner.split_once(')').ok_or_else(malformed)?;
    let reason = rest
        .trim_start()
        .strip_prefix(':')
        .ok_or_else(malformed)?
        .trim()
        .trim_end_matches("*/")
        .trim();

    if reason.is_empty() {
        bail!(
            "waiver missing a reason: `{}` — every waiver must say why the file is exempt",
            line.trim()
        );
    }

    let (location, coverage) = match scope.trim() {
        "location" => (true, false),
        "coverage" => (false, true),
        "all" => (true, true),
        other => bail!("unknown waiver scope `{other}` — use `location`, `coverage`, or `all`"),
    };

    Ok(Waiver {
        location,
        coverage,
        reason: reason.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_location_waiver_with_its_reason() {
        let waivers =
            parse_waivers("// testing-conventions:waiver(location): thin CLI launcher").unwrap();
        assert_eq!(waivers.len(), 1);
        assert!(waivers[0].covers(Scope::Location));
        assert!(!waivers[0].covers(Scope::Coverage));
        assert_eq!(waivers[0].reason, "thin CLI launcher");
    }

    #[test]
    fn coverage_scope_covers_only_coverage() {
        let waivers =
            parse_waivers("# testing-conventions:waiver(coverage): generated stubs").unwrap();
        assert!(waivers[0].covers(Scope::Coverage));
        assert!(!waivers[0].covers(Scope::Location));
    }

    #[test]
    fn all_scope_covers_both() {
        let waivers =
            parse_waivers("// testing-conventions:waiver(all): vendored verbatim").unwrap();
        assert!(waivers[0].covers(Scope::Location));
        assert!(waivers[0].covers(Scope::Coverage));
    }

    #[test]
    fn trims_a_trailing_block_comment_close_from_the_reason() {
        let waivers =
            parse_waivers("/* testing-conventions:waiver(location): boilerplate shim */").unwrap();
        assert_eq!(waivers[0].reason, "boilerplate shim");
    }

    #[test]
    fn a_file_with_no_marker_has_no_waivers() {
        assert!(parse_waivers("export const x = 1;\n// ordinary comment\n")
            .unwrap()
            .is_empty());
    }

    #[test]
    fn an_empty_reason_is_an_error_not_a_silent_pass() {
        assert!(parse_waivers("// testing-conventions:waiver(location):").is_err());
        assert!(parse_waivers("// testing-conventions:waiver(location):    ").is_err());
    }

    #[test]
    fn a_missing_scope_is_an_error() {
        assert!(parse_waivers("// testing-conventions:waiver: no scope").is_err());
        assert!(parse_waivers("// testing-conventions:waiver(): empty scope").is_err());
    }

    #[test]
    fn an_unknown_scope_is_an_error() {
        assert!(parse_waivers("// testing-conventions:waiver(typo): bad scope").is_err());
    }

    #[test]
    fn waived_reason_reports_the_reason_for_a_covered_scope() {
        let source = "# testing-conventions:waiver(coverage): launcher only run in prod";
        assert_eq!(
            waived_reason(source, Scope::Coverage).unwrap().as_deref(),
            Some("launcher only run in prod"),
        );
        assert_eq!(waived_reason(source, Scope::Location).unwrap(), None);
    }

    #[test]
    fn a_malformed_marker_errors_even_when_querying_an_unrelated_scope() {
        // A broken coverage waiver must not be swallowed by a location query.
        let source = "// testing-conventions:waiver(coverage):";
        assert!(waived_reason(source, Scope::Location).is_err());
    }

    #[test]
    fn tolerates_whitespace_around_the_structure() {
        let waivers =
            parse_waivers("//   testing-conventions:waiver(  all  ) :   spaced out  ").unwrap();
        assert!(waivers[0].covers(Scope::Location));
        assert!(waivers[0].covers(Scope::Coverage));
        assert_eq!(waivers[0].reason, "spaced out");
    }
}
