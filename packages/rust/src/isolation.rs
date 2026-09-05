//! Rust unit-isolation lint: an inline `#[cfg(test)] mod` may call and import only into the
//! unit under test, its parent module reached via `super::`. The AST walk is the deterministic
//! `syn` heuristic; its design and precision limits live in `internals/rust/isolation.md`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use syn::spanned::Spanned;
use syn::visit::{self, Visit};

pub use crate::violation::Violation;

const RULE_CALL: &str = "no-out-of-module-call";
const RULE_IMPORT: &str = "no-out-of-module-import";
const RULE_DOUBLE: &str = "no-first-party-double";

/// The `unit lint` language selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Language {
    /// Inline `#[cfg(test)]` modules in `*.rs` files (`no-out-of-module-call`).
    #[value(name = "rust")]
    Rust,
    /// `*.test.{ts,tsx,mts,cts}` unit tests (`unmocked-collaborator`);
    /// the detector lives in [`crate::ts`].
    #[value(name = "typescript")]
    TypeScript,
    /// `*_test.py` / `test_*.py` colocated unit tests (`unmocked-collaborator`);
    /// the detector lives in [`crate::lint`].
    #[value(name = "python")]
    Python,
}

/// Every isolation violation in the unit source under crate root `root`, sorted by
/// `(file, line)`. `root`'s `Cargo.toml` names the external crates. `tests/`, `benches/`,
/// `examples/`, and `target/` are not unit source, so a local build changes no result.
pub fn find_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let deps = external_deps(root)?;

    let mut files = Vec::new();
    crate::colocated_test::collect_rust_source_files(root, &mut files)?;
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

/// Every `no-first-party-double` violation in the `tests/` crates under crate root `root`.
/// An integration test runs first-party code for real, so doubling it is the error;
/// doubling an external crate is fine.
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

/// Walks one integration-test file, flagging a `#[double]` of a first-party crate.
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

/// `true` for a `#[double]` / `#[mockall_double::double]` attribute.
fn has_double_attr(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path()
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "double")
    })
}

/// The crate's own `[package].name` plus every `path` dependency, hyphens normalized to
/// underscores. A `tests/` crate names the library under test by crate name rather than
/// `crate::`, so the name is what a `#[double]` import is matched against.
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

/// `true` when `file` (under `root`) is a Rust integration test — a `*.rs` file with a
/// `tests` component. An inline `#[cfg(test)]` unit test doubles its collaborators by
/// design; only a `tests/` crate runs first-party code for real.
fn is_integration_test(root: &Path, file: &Path) -> bool {
    file.strip_prefix(root)
        .unwrap_or(file)
        .components()
        .any(|component| component.as_os_str() == "tests")
}

/// Walks one parsed file, flagging out-of-module calls inside `#[cfg(test)]` modules.
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

/// Why a call's leading path is out-of-module, or `None` when it stays in-module or is
/// unresolvable — an unresolvable path is not flagged, the `syn` heuristic's known limit.
fn classify(path: &syn::Path, deps: &BTreeSet<String>) -> Option<&'static str> {
    let segs: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
    match segs.first().map(String::as_str)? {
        "self" | "Self" => None,
        "super" => (segs.get(1).map(String::as_str) == Some("super")).then_some("ancestor module"),
        "crate" => Some("first-party module"),
        "std" => is_effectful_std(&segs).then_some("effectful std"),
        // `core`/`alloc` carry no effectful APIs.
        "core" | "alloc" => None,
        // A local type or fn, including one imported by `super::*`, is in-module.
        other => deps.contains(other).then_some("external crate"),
    }
}

/// `true` for an effectful `std` path — fs, net, process, env, threads, OS, the clock, or
/// real-handle I/O. Pure `std` stays in-module: `internals/rust/testing.md` makes
/// `io::Cursor` the idiomatic in-memory unit-test tool.
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
/// `([a, b], false)` and `([a, c], true)`. A rename is judged by its source path.
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

/// Why a `use` reaches out of the test's own module, or `None` when it stays in-module.
/// The one legal glob is `super::*`; a named import is judged by its root like a call.
fn classify_use(segs: &[String], is_glob: bool, deps: &BTreeSet<String>) -> Option<&'static str> {
    match segs.first().map(String::as_str)? {
        "super" => (segs.get(1).map(String::as_str) == Some("super")).then_some("ancestor module"),
        "self" | "Self" => None,
        "crate" => Some("first-party module"),
        "std" if is_effectful_std(segs) => Some("effectful std"),
        // A glob of anything but `super` is foreign, even for pure `std`.
        "std" | "core" | "alloc" => is_glob.then_some("glob import"),
        other => {
            if deps.contains(other) {
                Some("external crate")
            } else {
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

/// Render a path back to `a::b::c` for the message; generic args are dropped.
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

/// `true` when `attrs` carries a `#[cfg(test)]` gate, including `cfg(all(test, …))` and
/// `cfg(any(test, …))` — the signal for an inline unit-test module.
pub(crate) fn has_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("cfg")
            && attr
                .meta
                .require_list()
                .map(|list| cfg_mentions_test(list.tokens.clone()))
                .unwrap_or(false)
    })
}

/// `true` when a `cfg(...)` predicate positively requires `test`. `#[cfg(not(test))]` gates
/// production code for non-test builds, and a `feature = "test"` string never counts.
fn cfg_mentions_test(tokens: proc_macro2::TokenStream) -> bool {
    cfg_requires_test(tokens, false)
}

/// `true` when a bare `test` ident is reached under an even number of enclosing `not(...)`
/// groups. `negated` flips inside each `not(...)`, so `not(test)` does not qualify.
fn cfg_requires_test(tokens: proc_macro2::TokenStream, negated: bool) -> bool {
    let mut iter = tokens.into_iter().peekable();
    while let Some(tt) = iter.next() {
        match tt {
            proc_macro2::TokenTree::Ident(id) if id == "not" => {
                // `not` applies to the group immediately following it.
                if let Some(proc_macro2::TokenTree::Group(group)) = iter.peek() {
                    let stream = group.stream();
                    iter.next();
                    if cfg_requires_test(stream, !negated) {
                        return true;
                    }
                }
            }
            proc_macro2::TokenTree::Ident(id) => {
                if !negated && id == "test" {
                    return true;
                }
            }
            proc_macro2::TokenTree::Group(group) if cfg_requires_test(group.stream(), negated) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

/// The crate's `[dependencies]` names, hyphens normalized to underscores — the external
/// crates whose calls are out-of-module. `[dev-dependencies]` are excluded: a unit test
/// uses its framework (`mockall`, `rstest`, …) for real.
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
    use std::sync::atomic::{AtomicU64, Ordering};

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
        assert!(is_effectful_std(&segs("std::fs::read")));
        assert!(is_effectful_std(&segs("std::net::TcpStream::connect")));
        assert!(is_effectful_std(&segs("std::env::var")));
        assert!(is_effectful_std(&segs("std::process::exit")));
        assert!(is_effectful_std(&segs("std::thread::sleep")));
        assert!(is_effectful_std(&segs("std::time::SystemTime::now")));
        assert!(is_effectful_std(&segs("std::io::stdout")));
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
        assert!(!has_cfg_test(&module("#[cfg(not(test))] mod t {}").attrs));
        assert!(!has_cfg_test(
            &module("#[cfg(all(not(test), unix))] mod t {}").attrs
        ));
        assert!(!has_cfg_test(
            &module("#[cfg(not(all(test, unix)))] mod t {}").attrs
        ));
        assert!(has_cfg_test(
            &module("#[cfg(not(not(test)))] mod t {}").attrs
        ));
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
        // Flagged: the crate glob, the crate named import, `rand`, and `std::fs`.
        let violations = violations_in(src, &["rand"]);
        assert_eq!(violations.len(), 4, "got {violations:?}");
        assert!(violations.iter().all(|v| v.rule == RULE_IMPORT));
    }

    #[test]
    fn classify_use_roots() {
        let deps: BTreeSet<String> = ["rand"].iter().map(|s| s.to_string()).collect();
        let segs = |p: &str| p.split("::").map(str::to_string).collect::<Vec<_>>();
        assert_eq!(classify_use(&segs("super"), true, &deps), None); // `use super::*`
        assert_eq!(classify_use(&segs("super::Thing"), false, &deps), None);
        assert_eq!(classify_use(&segs("self::helper"), false, &deps), None);
        assert_eq!(
            classify_use(&segs("std::collections::HashMap"), false, &deps),
            None
        );
        assert_eq!(classify_use(&segs("std::io::Cursor"), false, &deps), None);
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
        // Only `widget` is first-party: `rand` is external and `crate::` is the test crate.
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

    struct TempTree(PathBuf);

    impl TempTree {
        fn new(files: &[(&str, &str)]) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tc-isolation-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            for (rel, content) in files {
                let path = root.join(rel);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, content).unwrap();
            }
            std::fs::create_dir_all(&root).unwrap();
            TempTree(root)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_tree_without_a_manifest_resolves_to_empty_crate_sets() {
        let tree = TempTree::new(&[("src/lib.rs", "fn run() {}\n")]);
        assert!(first_party_crates(tree.path()).unwrap().is_empty());
        assert!(external_deps(tree.path()).unwrap().is_empty());
    }

    #[test]
    fn a_path_dependency_is_first_party_and_a_registry_one_is_not() {
        let tree = TempTree::new(&[(
            "Cargo.toml",
            "[package]\n\
             name = \"my-crate\"\n\n\
             [dependencies]\n\
             sibling-lib = { path = \"../sibling-lib\" }\n\
             rand = \"0.8\"\n\n\
             [dev-dependencies]\n\
             test-support = { path = \"../test-support\" }\n\
             mockall = \"0.13\"\n",
        )]);

        let first_party = first_party_crates(tree.path()).unwrap();
        assert_eq!(
            first_party,
            ["my_crate", "sibling_lib", "test_support"]
                .iter()
                .map(|s| (*s).to_string())
                .collect::<BTreeSet<String>>(),
            "the crate's own name and every path dep, hyphens normalized"
        );

        let external = external_deps(tree.path()).unwrap();
        assert_eq!(
            external,
            ["rand", "sibling_lib"]
                .iter()
                .map(|s| (*s).to_string())
                .collect::<BTreeSet<String>>(),
            "`[dependencies]` only — a dev-dependency is test tooling, not a collaborator"
        );
    }

    #[test]
    fn a_call_through_a_non_path_callee_is_left_alone() {
        let src = "\
#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let _ = (make())(1);
    }
}
";
        assert!(
            violations_in(src, &["rand"]).is_empty(),
            "a callee that is not a path carries no leading segment to classify"
        );
    }

    #[test]
    fn a_renamed_import_is_judged_by_its_source_path() {
        let src = "\
#[cfg(test)]
mod tests {
    use crate::other::Thing as Local;
    use super::Widget as W;
}
";
        let violations = violations_in(src, &[]);
        assert_eq!(violations.len(), 1, "got {violations:?}");
        let m = &violations[0].message;
        assert!(
            m.contains("crate::other::Thing"),
            "the message names the source path, not the alias: {m}"
        );
    }

    #[test]
    fn a_grouped_import_is_flattened_leaf_by_leaf() {
        let src = "\
#[cfg(test)]
mod tests {
    use crate::other::{Named, deeper::Other};
    use super::{Widget, helper};
}
";
        let violations = violations_in(src, &[]);
        assert_eq!(violations.len(), 2, "got {violations:?}");
        let (first, second) = (&violations[0].message, &violations[1].message);
        assert!(first.contains("crate::other::Named"), "{first}");
        assert!(second.contains("crate::other::deeper::Other"), "{second}");
    }

    #[test]
    fn a_glob_of_an_unresolvable_root_is_still_a_glob_import() {
        let deps: BTreeSet<String> = ["rand"].iter().map(|s| s.to_string()).collect();
        let segs = |p: &str| p.split("::").map(str::to_string).collect::<Vec<_>>();
        assert_eq!(
            classify_use(&segs("helpers"), true, &deps),
            Some("glob import"),
            "a glob is foreign even when `syn` cannot resolve its root"
        );
        assert_eq!(
            classify_use(&segs("helpers::Thing"), false, &deps),
            None,
            "a named import of an unresolvable root is the heuristic's documented limit"
        );
    }

    #[test]
    fn a_leading_colon_survives_into_the_message() {
        let src = "\
#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let _ = ::std::fs::read(\"x\");
    }
}
";
        let violations = violations_in(src, &[]);
        assert_eq!(violations.len(), 1, "got {violations:?}");
        let m = &violations[0].message;
        assert!(m.contains("`::std::fs::read`"), "{m}");
    }

    #[test]
    fn a_bare_cfg_not_is_not_a_test_module() {
        let module = |s: &str| syn::parse_str::<syn::ItemMod>(s).expect("module parses");
        assert!(!has_cfg_test(&module("#[cfg(not)] mod t {}").attrs));
    }
}
