//! Changelog rule: a pull request that changes a package's public surface adds a fragment
//! recording it. The layout is discovered from where the fragment directories sit, so a
//! consumer declares nothing. Pure decisions here; the git reads are injected by the caller.

use std::path::Path;

/// Where a repository keeps its fragments, which decides what the check scopes findings to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// `packages/<pkg>/changelog.d/` — a fragment's owning package is its parent directory.
    PerPackage,
    /// A single `changelog.d/` outside any package — one fragment per pull request.
    Pooled,
}

/// The two fragment kinds, in the order findings report them missing.
pub const KINDS: [&str; 2] = ["changelog", "migrations"];

/// One thing the pull request owes. `file` is set when the finding belongs to a path, so the
/// annotation lands on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub file: Option<String>,
    pub message: String,
}

/// The fragment layout `root` uses, or `None` when it keeps no fragments and the check skips.
pub fn discover_layout(_root: &Path) -> Option<Layout> {
    todo!("discover_layout")
}

/// Whether `root` enforces migration fragments — true when it keeps a `migrations.d/`.
pub fn migrations_enforced(_root: &Path) -> bool {
    todo!("migrations_enforced")
}

/// Whether `name` matches `YYYY-MM-DD-<slug>.md`: the UTC merge date, then a kebab-case slug.
pub fn fragment_name_ok(_name: &str) -> bool {
    todo!("fragment_name_ok")
}

/// Whether any commit body carries a line opening with `skip-changelog:`.
pub fn has_skip_line(_bodies: &str) -> bool {
    todo!("has_skip_line")
}

/// Whether `path` is something `pkg` may change without owing a fragment.
pub fn is_exempt(_path: &str, _pkg: &str) -> bool {
    todo!("is_exempt")
}

/// The package directories the paths touch, unique and sorted.
pub fn changed_packages(_changed: &[String]) -> Vec<String> {
    todo!("changed_packages")
}

/// Everything the pull request owes: a malformed fragment name, or a scope that changed public
/// surface without adding the fragments that record it.
pub fn findings(
    _layout: Layout,
    _migrations: bool,
    _changed: &[String],
    _added: &[String],
) -> Vec<Finding> {
    todo!("findings")
}
