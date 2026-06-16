//! Rust unit-isolation lint (#44): an inline `#[cfg(test)] mod` may call only into
//! the unit under test — its parent module, reached via `super::`. A call *out of
//! the test's own module* — into another first-party module (`crate::…`), an
//! external crate, or effectful `std` — is a violation. Inject a trait double
//! (hand-rolled or `mockall`) instead; the compiler checks the double.
//!
//! Detection is AST-based: each `*.rs` file under the crate root is parsed with
//! `syn` and its `#[cfg(test)]` modules are walked. This is the deterministic
//! `syn` heuristic; full name-resolution precision is a future `dylint` pass. The
//! design and its precision limits live in `internals/rust/isolation.md`.
//!
//! Skeleton: this commit wires the file walk + `syn` parse and reports nothing;
//! the D1 detector (out-of-module path calls) lands next.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

pub use crate::violation::Violation;

/// A language whose unit-isolation convention can be checked. Rust only for now
/// (Python #42 / TypeScript #43 are separate detectors).
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Language {
    /// Inline `#[cfg(test)]` modules in `*.rs` files.
    #[value(name = "rust")]
    Rust,
}

/// Scan the Rust source files under `root` and return every isolation violation,
/// sorted by `(file, line)` for deterministic output.
///
/// `root` is the crate root (its `Cargo.toml` names the external crates). Every
/// `*.rs` file under it is parsed; a file that cannot be read or parsed is an
/// error.
pub fn find_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_rust_files(root, &mut files)?;
    files.sort();

    let mut violations: Vec<Violation> = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading source file `{}`", file.display()))?;
        syn::parse_file(&source).map_err(|err| anyhow!("parsing `{}`: {err}", file.display()))?;
        // Skeleton: detection lands in the D1 slice.
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
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
