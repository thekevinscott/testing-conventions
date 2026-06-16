//! Rust unit-isolation lint (#44): an inline `#[cfg(test)] mod` may call only into
//! the unit under test — its parent module, reached via `super::`. A call *out of
//! the test's own module* — into another first-party module (`crate::…`), an
//! external crate, or effectful `std` — is a violation. Inject a trait double
//! (hand-rolled or `mockall`) instead; the compiler checks the double.
//!
//! Detection is AST-based: each `*.rs` file under the crate root is parsed with
//! `syn` and its `#[cfg(test)]` modules are walked with a [`Visit`]or. This is the
//! deterministic `syn` heuristic; full name-resolution precision is a future
//! `dylint` pass. The design and its precision limits live in
//! `internals/rust/isolation.md`.
//!
//! Implemented detectors:
//! - **`no-out-of-module-call`** (D1): a call expression `A::…::f(…)` inside a
//!   `#[cfg(test)]` module whose leading segment `A` reaches out of the module —
//!   `crate::` (first-party, another module), `super::super::…` (an ancestor),
//!   an external crate from `Cargo.toml`, or effectful `std`. A single `super::`,
//!   `self`/`Self`, a bare/unqualified call, and pure `std` (incl. `io::Cursor`)
//!   stay in-module and are not flagged.
//! - **`no-out-of-module-import`** (D2): a `use` inside a `#[cfg(test)]` module
//!   that brings in a foreign surface — a glob of anything but `super::*`, or a
//!   named import rooted at `crate::`, an external crate, or effectful `std`.
//!   `use super::*` / `use super::Thing` (the unit under test), `self`, and pure
//!   `std` (e.g. `collections`, `io::Cursor`) are in-module. Catches a collaborator
//!   imported then called unqualified, which D1's call check can't see.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

pub use crate::violation::Violation;

/// Rule id reported for an out-of-module call (D1).
const RULE_CALL: &str = "no-out-of-module-call";
/// Rule id reported for an out-of-module `use` import (D2).
const RULE_IMPORT: &str = "no-out-of-module-import";
/// Rule id reported for doubling a first-party item in an integration test.
const RULE_DOUBLE: &str = "no-first-party-double";

/// A language whose unit-isolation convention can be checked (Python #42 is a
/// separate detector). Each detector lives in its own module; this enum is the
/// shared `unit isolation` language selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Language {
    /// Inline `#[cfg(test)]` modules in `*.rs` files (`no-out-of-module-call`).
    #[value(name = "rust")]
    Rust,
    /// `*.test.{ts,tsx,mts,cts}` unit tests (`unmocked-collaborator`, #43 / #76);
    /// the detector lives in [`crate::ts`].
    #[value(name = "typescript")]
    TypeScript,
}

/// Scan the Rust source files under `root` and return every isolation violation,
/// sorted by `(file, line)` for deterministic output.
///
/// `root` is the crate root: its `Cargo.toml` names the external crates whose
/// calls are out-of-module. Every `*.rs` file under it is parsed; a file that
/// cannot be read or parsed is an error.
pub fn find_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let deps = external_deps(root)?;

    let mut files = Vec::new();
    collect_rust_files(root, &mut files)?;
    files.sort();

    let mut violations = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading source file `{}`", file.display()))?;
        let ast = syn::parse_file(&source)
            .map_err(|err| anyhow!("parsing `{}`: {err}", file.display()))?;
        let mut visitor = IsolationVisitor {
            file,
            deps: &deps,
            test_depth: 0,
            violations: Vec::new(),
        };
        visitor.visit_file(&ast);
        violations.append(&mut visitor.violations);
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// Scan the Rust integration crates under `root` (the `*.rs` files in a `tests/`
/// directory) and return every `no-first-party-double` violation — a `#[double]`
/// import of a first-party item. An integration test runs first-party code for
/// real, so doubling it is the error; doubling an external crate is fine. `root`
/// is the crate root; its `Cargo.toml` names the first-party crates.
pub fn find_integration_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let first_party = first_party_crates(root)?;

    let mut files = Vec::new();
    collect_rust_files(root, &mut files)?;
    files.retain(|file| is_integration_test(root, file));
    files.sort();

    let mut violations = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading source file `{}`", file.display()))?;
        let ast = syn::parse_file(&source)
            .map_err(|err| anyhow!("parsing `{}`: {err}", file.display()))?;
        let mut visitor = DoubleVisitor {
            file,
            first_party: &first_party,
            violations: Vec::new(),
        };
        visitor.visit_file(&ast);
        violations.append(&mut visitor.violations);
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// Walks one parsed integration-test file, flagging a `#[double]` import whose
/// path names a first-party crate.
struct DoubleVisitor<'a> {
    file: &'a Path,
    first_party: &'a BTreeSet<String>,
    violations: Vec<Violation>,
}

impl<'ast> Visit<'ast> for DoubleVisitor<'_> {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        if has_double_attr(&node.attrs) {
            let mut imports = Vec::new();
            flatten_use(&node.tree, &mut Vec::new(), &mut imports);
            // One finding per `#[double] use`: flag if any leaf is first-party.
            if let Some((segs, is_glob)) = imports.iter().find(|(segs, _)| {
                segs.first()
                    .is_some_and(|root| self.first_party.contains(root))
            }) {
                self.violations.push(Violation {
                    file: self.file.to_path_buf(),
                    line: node.span().start().line,
                    rule: RULE_DOUBLE,
                    message: format!(
                        "integration test doubles first-party `{}` with `#[double]`; \
                         run first-party code for real — only external crates may be doubled",
                        render_use(segs, *is_glob),
                    ),
                });
            }
        }
        visit::visit_item_use(self, node);
    }
}

/// `true` when `attrs` carries a `#[double]` (or `#[mockall_double::double]`)
/// attribute — `mockall_double` swapping a real item for its mock.
fn has_double_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "double")
    })
}

/// The crate's first-party crates: its own `[package].name` plus every `path`
/// dependency (your own crates, run for real), hyphens normalized to underscores.
/// In a `tests/` integration crate the library under test is referenced by its
/// crate name (not `crate::`, which is the test crate itself). Registry deps —
/// including `mockall` / `mockall_double` — are external and absent here. Empty
/// when there is no `Cargo.toml` at `root`.
fn first_party_crates(root: &Path) -> Result<BTreeSet<String>> {
    let manifest = root.join("Cargo.toml");
    let mut set = BTreeSet::new();
    if !manifest.is_file() {
        return Ok(set);
    }
    let text = std::fs::read_to_string(&manifest)
        .with_context(|| format!("reading `{}`", manifest.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("parsing `{}`", manifest.display()))?;

    if let Some(name) = value
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
    {
        set.insert(name.replace('-', "_"));
    }
    for table_name in ["dependencies", "dev-dependencies"] {
        if let Some(table) = value.get(table_name).and_then(toml::Value::as_table) {
            for (name, spec) in table {
                if spec.as_table().is_some_and(|t| t.contains_key("path")) {
                    set.insert(name.replace('-', "_"));
                }
            }
        }
    }
    Ok(set)
}

/// `true` when `file` (under `root`) is a Rust integration test — a `*.rs` file
/// with a `tests` directory in its `root`-relative path. Unit tests are inline
/// `#[cfg(test)]` in `src/`, where doubling a collaborator is correct isolation;
/// only `tests/` crates run first-party for real and so are integration subjects.
fn is_integration_test(root: &Path, file: &Path) -> bool {
    file.strip_prefix(root)
        .unwrap_or(file)
        .components()
        .any(|component| component.as_os_str() == "tests")
}

/// Walks one parsed file, flagging out-of-module calls inside `#[cfg(test)]`
/// modules. `test_depth` counts how deep we are inside such modules, so a call in
/// non-test code is ignored.
struct IsolationVisitor<'a> {
    file: &'a Path,
    deps: &'a BTreeSet<String>,
    test_depth: usize,
    violations: Vec<Violation>,
}

impl<'ast> Visit<'ast> for IsolationVisitor<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let is_test = has_cfg_test(&node.attrs);
        if is_test {
            self.test_depth += 1;
        }
        visit::visit_item_mod(self, node);
        if is_test {
            self.test_depth -= 1;
        }
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.test_depth > 0 {
            if let syn::Expr::Path(path_expr) = node.func.as_ref() {
                if let Some(kind) = classify(&path_expr.path, self.deps) {
                    self.violations.push(Violation {
                        file: self.file.to_path_buf(),
                        line: node.span().start().line,
                        rule: RULE_CALL,
                        message: format!(
                            "unit test calls `{}` out of its own module ({kind}); \
                             inject a trait double — only `super::` is in-module",
                            render_path(&path_expr.path),
                        ),
                    });
                }
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        if self.test_depth > 0 {
            let mut imports = Vec::new();
            flatten_use(&node.tree, &mut Vec::new(), &mut imports);
            for (segs, is_glob) in &imports {
                if let Some(kind) = classify_use(segs, *is_glob, self.deps) {
                    self.violations.push(Violation {
                        file: self.file.to_path_buf(),
                        line: node.span().start().line,
                        rule: RULE_IMPORT,
                        message: format!(
                            "unit test imports `{}` out of its own module ({kind}); \
                             only `super::` (the unit) and pure `std` belong in a unit test",
                            render_use(segs, *is_glob),
                        ),
                    });
                }
            }
        }
        visit::visit_item_use(self, node);
    }
}

/// Why a call's leading path is out-of-module, or `None` when the call stays
/// in-module (or is unresolvable, and so deliberately not flagged — the `syn`
/// heuristic's documented limit).
fn classify(path: &syn::Path, deps: &BTreeSet<String>) -> Option<&'static str> {
    let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    match segs.first().map(String::as_str)? {
        // `self` / `Self` are local; a single `super::` is the unit under test.
        "self" | "Self" => None,
        "super" => (segs.get(1).map(String::as_str) == Some("super")).then_some("ancestor module"),
        "crate" => Some("first-party module"),
        "std" => is_effectful_std(&segs).then_some("effectful std"),
        // `core`/`alloc` carry no effectful APIs.
        "core" | "alloc" => None,
        // Any other leading segment is in-module unless it names an external
        // crate; a local type/fn (incl. `super::*`-imported) is not flagged.
        other => deps.contains(other).then_some("external crate"),
    }
}

/// `true` for an effectful `std` path — filesystem, network, process, env,
/// threads, OS, the clock (`SystemTime::now` / `Instant::now`), or real-handle
/// I/O (`stdin`/`stdout`/`stderr`). Pure std is allowed: `std::io::Cursor` and the
/// I/O traits, `time::Duration`, `collections`, `fmt`, … — `internals/rust/`
/// `testing.md` makes `Cursor` the idiomatic in-memory unit-test tool.
fn is_effectful_std(segs: &[String]) -> bool {
    match segs.get(1).map(String::as_str) {
        Some("fs" | "net" | "process" | "env" | "thread" | "os") => true,
        Some("io") => matches!(
            segs.get(2).map(String::as_str),
            Some("stdin" | "stdout" | "stderr")
        ),
        Some("time") => {
            matches!(
                segs.get(2).map(String::as_str),
                Some("SystemTime" | "Instant")
            ) && segs.get(3).map(String::as_str) == Some("now")
        }
        _ => false,
    }
}

/// Flatten a `use` tree into `(path, is_glob)` leaves: `use a::{b, c::*}` yields
/// `([a, b], false)` and `([a, c], true)`. A rename (`use a::b as c`) is judged by
/// its source path `[a, b]`.
fn flatten_use(tree: &syn::UseTree, prefix: &mut Vec<String>, out: &mut Vec<(Vec<String>, bool)>) {
    match tree {
        syn::UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            flatten_use(&path.tree, prefix, out);
            prefix.pop();
        }
        syn::UseTree::Name(name) => {
            let mut full = prefix.clone();
            full.push(name.ident.to_string());
            out.push((full, false));
        }
        syn::UseTree::Rename(rename) => {
            let mut full = prefix.clone();
            full.push(rename.ident.to_string());
            out.push((full, false));
        }
        syn::UseTree::Glob(_) => out.push((prefix.clone(), true)),
        syn::UseTree::Group(group) => {
            for item in &group.items {
                flatten_use(item, prefix, out);
            }
        }
    }
}

/// Why a `use` import reaches out of the test's own module, or `None` when it
/// stays in-module. The one legal glob is `super::*`; any other glob is foreign. A
/// named import is judged by its root like a call — `crate::`, an external crate,
/// or effectful `std` are out; `super`/`self`, pure `std`, and a local name are in.
fn classify_use(segs: &[String], is_glob: bool, deps: &BTreeSet<String>) -> Option<&'static str> {
    match segs.first().map(String::as_str)? {
        // `super::*` / `super::Thing` are the unit under test; `super::super::…`
        // reaches past it.
        "super" => (segs.get(1).map(String::as_str) == Some("super")).then_some("ancestor module"),
        "self" | "Self" => None,
        "crate" => Some("first-party module"),
        "std" if is_effectful_std(segs) => Some("effectful std"),
        // Pure `std` / `core` / `alloc`: a named import is in-module, but a glob of
        // anything but `super` is foreign (the issue's bright line).
        "std" | "core" | "alloc" => is_glob.then_some("glob import"),
        other => {
            if deps.contains(other) {
                Some("external crate")
            } else {
                // A local module/type: a named import is in-module; a non-`super`
                // glob is still foreign.
                is_glob.then_some("glob import")
            }
        }
    }
}

/// Render a flattened import for the message: `a::b`, or `a::b::*` for a glob.
fn render_use(segs: &[String], is_glob: bool) -> String {
    let mut out = segs.join("::");
    if is_glob {
        if !out.is_empty() {
            out.push_str("::");
        }
        out.push('*');
    }
    out
}

/// Render a path back to `a::b::c` for the message (idents only; generic args
/// dropped).
fn render_path(path: &syn::Path) -> String {
    let mut out = String::new();
    if path.leading_colon.is_some() {
        out.push_str("::");
    }
    for (i, seg) in path.segments.iter().enumerate() {
        if i > 0 {
            out.push_str("::");
        }
        out.push_str(&seg.ident.to_string());
    }
    out
}

/// `true` when `attrs` carries a `#[cfg(test)]` gate (including `cfg(all(test, …))`
/// / `cfg(any(test, …))`) — the signal for an inline unit-test module.
fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .meta
                .require_list()
                .map(|list| cfg_mentions_test(list.tokens.clone()))
                .unwrap_or(false)
    })
}

/// `true` when a `cfg(...)` token stream contains a bare `test` ident (recursing
/// into `all(...)` / `any(...)` groups). A `feature = "test"` string literal does
/// not count.
fn cfg_mentions_test(tokens: proc_macro2::TokenStream) -> bool {
    tokens.into_iter().any(|tt| match tt {
        proc_macro2::TokenTree::Ident(id) => id == "test",
        proc_macro2::TokenTree::Group(group) => cfg_mentions_test(group.stream()),
        _ => false,
    })
}

/// The crate's normal `[dependencies]` names (hyphens normalized to underscores,
/// the form used in paths) — the external crates whose calls are out-of-module.
/// `[dev-dependencies]` are test tooling (`mockall`, `rstest`, …) and are
/// deliberately excluded: a unit test uses its framework for real. Returns an
/// empty set when there is no `Cargo.toml` at `root`.
fn external_deps(root: &Path) -> Result<BTreeSet<String>> {
    let manifest = root.join("Cargo.toml");
    if !manifest.is_file() {
        return Ok(BTreeSet::new());
    }
    let text = std::fs::read_to_string(&manifest)
        .with_context(|| format!("reading `{}`", manifest.display()))?;
    let value: toml::Value =
        toml::from_str(&text).with_context(|| format!("parsing `{}`", manifest.display()))?;
    let mut deps = BTreeSet::new();
    if let Some(table) = value.get("dependencies").and_then(toml::Value::as_table) {
        for name in table.keys() {
            deps.insert(name.replace('-', "_"));
        }
    }
    Ok(deps)
}

/// Recursively collect every `*.rs` file under `dir` into `out`.
fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_rust_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run the visitor over a source snippet with the given external-crate deps.
    fn violations_in(src: &str, deps: &[&str]) -> Vec<Violation> {
        let ast = syn::parse_file(src).expect("snippet parses");
        let dep_set: BTreeSet<String> = deps.iter().map(|s| (*s).to_string()).collect();
        let mut visitor = IsolationVisitor {
            file: Path::new("snippet.rs"),
            deps: &dep_set,
            test_depth: 0,
            violations: Vec::new(),
        };
        visitor.visit_file(&ast);
        visitor.violations
    }

    #[test]
    fn flags_each_out_of_module_form() {
        let src = "\
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn t() {
        let _ = crate::store::load();
        let _ = std::fs::read(\"x\");
        let _ = rand::random::<u8>();
        let _ = super::super::util::help();
    }
}
";
        let violations = violations_in(src, &["rand"]);
        assert_eq!(violations.len(), 4, "got {violations:?}");
        assert!(violations.iter().all(|v| v.rule == RULE_CALL));
    }

    #[test]
    fn allows_in_module_calls() {
        let src = "\
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn t() {
        let _ = super::widget();
        let _ = self::helper();
        let _ = Cursor::new(b\"x\");
        let _ = std::collections::HashMap::<u8, u8>::new();
        assert_eq!(1, 1);
    }
}
";
        assert!(violations_in(src, &["rand"]).is_empty());
    }

    #[test]
    fn ignores_calls_outside_test_modules() {
        let src = "fn run() { let _ = crate::other::go(); }";
        assert!(violations_in(src, &[]).is_empty());
    }

    #[test]
    fn reports_the_call_line() {
        // Line 1 is `#[cfg(test)]`; the flagged call sits on line 4.
        let src = "\
#[cfg(test)]
mod tests {
    fn t() {
        let _ = crate::other::go();
    }
}
";
        let violations = violations_in(src, &[]);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].line, 4);
    }

    #[test]
    fn effectful_std_policy() {
        let segs = |p: &str| p.split("::").map(str::to_string).collect::<Vec<_>>();
        // effectful — flagged
        assert!(is_effectful_std(&segs("std::fs::read")));
        assert!(is_effectful_std(&segs("std::net::TcpStream::connect")));
        assert!(is_effectful_std(&segs("std::env::var")));
        assert!(is_effectful_std(&segs("std::process::exit")));
        assert!(is_effectful_std(&segs("std::thread::sleep")));
        assert!(is_effectful_std(&segs("std::time::SystemTime::now")));
        assert!(is_effectful_std(&segs("std::io::stdout")));
        // pure — allowed
        assert!(!is_effectful_std(&segs("std::collections::HashMap")));
        assert!(!is_effectful_std(&segs("std::io::Cursor")));
        assert!(!is_effectful_std(&segs("std::time::Duration")));
        assert!(!is_effectful_std(&segs("std::cmp::min")));
    }

    #[test]
    fn classify_leading_segment() {
        let deps: BTreeSet<String> = ["rand"].iter().map(|s| s.to_string()).collect();
        let path = |s: &str| syn::parse_str::<syn::Path>(s).expect("path parses");
        assert_eq!(classify(&path("super::foo"), &deps), None);
        assert_eq!(classify(&path("self::foo"), &deps), None);
        assert_eq!(classify(&path("Local::new"), &deps), None);
        assert_eq!(
            classify(&path("super::super::foo"), &deps),
            Some("ancestor module")
        );
        assert_eq!(
            classify(&path("crate::a::b"), &deps),
            Some("first-party module")
        );
        assert_eq!(
            classify(&path("rand::random"), &deps),
            Some("external crate")
        );
        assert_eq!(
            classify(&path("std::fs::read"), &deps),
            Some("effectful std")
        );
        assert_eq!(classify(&path("std::io::Cursor"), &deps), None);
    }

    #[test]
    fn recognizes_cfg_test_attribute() {
        let module = |s: &str| syn::parse_str::<syn::ItemMod>(s).expect("module parses");
        assert!(has_cfg_test(&module("#[cfg(test)] mod t {}").attrs));
        assert!(has_cfg_test(
            &module("#[cfg(all(test, feature = \"x\"))] mod t {}").attrs
        ));
        assert!(!has_cfg_test(
            &module("#[cfg(feature = \"test\")] mod t {}").attrs
        ));
        assert!(!has_cfg_test(&module("mod t {}").attrs));
    }

    #[test]
    fn flags_each_foreign_import() {
        let src = "\
#[cfg(test)]
mod tests {
    use super::*;
    use super::Thing;
    use crate::other::*;
    use crate::other::Named;
    use rand::Rng;
    use std::fs;
    use std::collections::HashMap;
    use std::io::Cursor;
}
";
        // Flagged: the crate glob, the crate named import, the external crate, and
        // effectful `std::fs` — not `super::*` / `super::Thing` / pure std.
        let violations = violations_in(src, &["rand"]);
        assert_eq!(violations.len(), 4, "got {violations:?}");
        assert!(violations.iter().all(|v| v.rule == RULE_IMPORT));
    }

    #[test]
    fn classify_use_roots() {
        let deps: BTreeSet<String> = ["rand"].iter().map(|s| s.to_string()).collect();
        let segs = |p: &str| p.split("::").map(str::to_string).collect::<Vec<_>>();
        // in-module (None)
        assert_eq!(classify_use(&segs("super"), true, &deps), None); // `use super::*`
        assert_eq!(classify_use(&segs("super::Thing"), false, &deps), None);
        assert_eq!(classify_use(&segs("self::helper"), false, &deps), None);
        assert_eq!(
            classify_use(&segs("std::collections::HashMap"), false, &deps),
            None
        );
        assert_eq!(classify_use(&segs("std::io::Cursor"), false, &deps), None);
        // out-of-module
        assert_eq!(
            classify_use(&segs("super::super"), true, &deps),
            Some("ancestor module")
        );
        assert_eq!(
            classify_use(&segs("crate::other"), true, &deps),
            Some("first-party module")
        );
        assert_eq!(
            classify_use(&segs("crate::other::Named"), false, &deps),
            Some("first-party module")
        );
        assert_eq!(
            classify_use(&segs("rand::Rng"), false, &deps),
            Some("external crate")
        );
        assert_eq!(
            classify_use(&segs("std::fs"), false, &deps),
            Some("effectful std")
        );
        // a non-`super` glob is foreign even for pure std
        assert_eq!(
            classify_use(&segs("std::collections"), true, &deps),
            Some("glob import")
        );
    }

    #[test]
    fn imports_outside_test_modules_are_ignored() {
        let src = "use crate::other::*; fn run() {}";
        assert!(violations_in(src, &[]).is_empty());
    }

    /// Run the `#[double]` detector over an integration-test snippet.
    fn integration_violations_in(src: &str, first_party: &[&str]) -> Vec<Violation> {
        let ast = syn::parse_file(src).expect("snippet parses");
        let set: BTreeSet<String> = first_party.iter().map(|s| (*s).to_string()).collect();
        let mut visitor = DoubleVisitor {
            file: Path::new("integration.rs"),
            first_party: &set,
            violations: Vec::new(),
        };
        visitor.visit_file(&ast);
        visitor.violations
    }

    #[test]
    fn flags_double_of_first_party_only() {
        let src = "\
use mockall_double::double;
#[double]
use widget::Renderer;
#[double]
use rand::rngs::ThreadRng;
#[double]
use crate::support::Helper;
";
        // Only the first-party `widget` double is flagged; `rand` (external) and
        // `crate::` (the test crate itself, not the library under test) are not.
        let violations = integration_violations_in(src, &["widget"]);
        assert_eq!(violations.len(), 1, "got {violations:?}");
        assert_eq!(violations[0].rule, RULE_DOUBLE);
    }

    #[test]
    fn ignores_use_without_double() {
        let src = "use widget::Renderer; fn t() {}";
        assert!(integration_violations_in(src, &["widget"]).is_empty());
    }

    #[test]
    fn recognizes_double_attribute() {
        let item = |s: &str| syn::parse_str::<syn::ItemUse>(s).expect("use parses");
        assert!(has_double_attr(&item("#[double] use a::B;").attrs));
        assert!(has_double_attr(
            &item("#[mockall_double::double] use a::B;").attrs
        ));
        assert!(!has_double_attr(
            &item("#[allow(unused_imports)] use a::B;").attrs
        ));
        assert!(!has_double_attr(&item("use a::B;").attrs));
    }
}
