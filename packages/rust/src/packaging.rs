//! Packaging rule — foundation (issue #70).
//!
//! README "Packaging": test files never ship in the built artifact. Colocated
//! unit tests live next to the source, so packaging has to strip them — and this
//! rule confirms it did, by inspecting the *built* artifact rather than the
//! working tree.
//!
//! This module is the deterministic core: given the root of an unpacked built
//! artifact and the test-file globs that must not appear in it, [`scan`] walks
//! the tree and returns every offending file. Producing the artifact (building a
//! wheel/sdist, `npm pack`, `cargo package`, then unpacking it) is a per-language
//! layer on top — kept separate, and out of this foundation slice, so the core
//! guarantee is testable without any language toolchain. The per-language slices
//! supply the build step and the glob set: Python `*_test.py` (#72), TypeScript
//! `*.test.*` (#73), Rust `tests/` (#74).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Walk `root` — the root of an unpacked built artifact — and return every file
/// whose name matches one of `globs`, sorted for deterministic output.
///
/// `globs` are file-name globs where `*` matches any run of characters
/// (including none); each is matched against an entry's file name, not its full
/// path. A non-empty result means test files leaked into the artifact. Returns
/// an error if the tree under `root` cannot be read.
pub fn scan(root: impl AsRef<Path>, globs: &[String]) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();
    let mut offenders = Vec::new();
    collect_offenders(root, globs, &mut offenders)?;
    offenders.sort();
    Ok(offenders)
}

/// Recursively collect every file under `dir` whose name matches one of `globs`.
fn collect_offenders(dir: &Path, globs: &[String], out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_offenders(&path, globs, out)?;
        } else if matches_any(&path, globs) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` when the file name of `path` matches any glob in `globs`.
fn matches_any(path: &Path, globs: &[String]) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    globs.iter().any(|glob| matches_glob(glob, name))
}

/// Match `name` against a file-name `glob` where `*` matches any run of
/// characters (including none) and every other character is literal.
///
/// `*` is the only metacharacter — it is all the test-file patterns this rule
/// checks (`*_test.py`, `*.test.*`) need. Matching is over Unicode scalar values.
fn matches_glob(glob: &str, name: &str) -> bool {
    let glob: Vec<char> = glob.chars().collect();
    let name: Vec<char> = name.chars().collect();
    // Linear wildcard match: walk `name`, and on a mismatch backtrack to the most
    // recent `*`, extending what it consumed by one character.
    let (mut g, mut n) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut consumed_by_star = 0usize;
    while n < name.len() {
        if g < glob.len() && glob[g] == name[n] {
            g += 1;
            n += 1;
        } else if g < glob.len() && glob[g] == '*' {
            star = Some(g);
            consumed_by_star = n;
            g += 1;
        } else if let Some(star) = star {
            // Mismatch under an open `*`: let the star swallow one more char.
            g = star + 1;
            consumed_by_star += 1;
            n = consumed_by_star;
        } else {
            return false;
        }
    }
    // The pattern matches iff what's left is only trailing `*`s (each empty).
    while g < glob.len() && glob[g] == '*' {
        g += 1;
    }
    g == glob.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A throwaway directory tree, removed on drop.
    struct TempTree(PathBuf);

    impl TempTree {
        fn new(files: &[&str]) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tc-packaging-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            for rel in files {
                let path = root.join(rel);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, "x").unwrap();
            }
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
    fn star_matches_any_run_including_empty() {
        assert!(matches_glob("*", ""));
        assert!(matches_glob("*", "anything.py"));
        // The `*` consumes nothing: the literal `.py` matches the whole name.
        assert!(matches_glob("*.py", ".py"));
    }

    #[test]
    fn the_python_test_glob_matches_only_test_files() {
        assert!(matches_glob("*_test.py", "widget_test.py"));
        assert!(!matches_glob("*_test.py", "widget.py"));
        // A trailing extension beyond `.py` must not match (no `*` at the end).
        assert!(!matches_glob("*_test.py", "widget_test.pyc"));
    }

    #[test]
    fn the_typescript_test_glob_matches_across_extensions() {
        assert!(matches_glob("*.test.*", "button.test.ts"));
        assert!(matches_glob("*.test.*", "button.test.mts"));
        assert!(matches_glob("*.test.*", "button.test.tsx"));
        assert!(!matches_glob("*.test.*", "button.ts"));
    }

    #[test]
    fn a_literal_glob_must_match_exactly() {
        assert!(matches_glob("conftest.py", "conftest.py"));
        assert!(!matches_glob("conftest.py", "conftest.pyi"));
        assert!(!matches_glob("conftest.py", "xconftest.py"));
    }

    #[test]
    fn scan_flags_a_test_file_anywhere_in_the_tree() {
        let tree = TempTree::new(&["pkg/widget.py", "pkg/sub/helper_test.py"]);
        let offenders = scan(tree.path(), &["*_test.py".to_string()]).unwrap();
        assert_eq!(offenders, vec![tree.path().join("pkg/sub/helper_test.py")]);
    }

    #[test]
    fn scan_is_clean_when_nothing_matches() {
        let tree = TempTree::new(&["pkg/widget.py", "pkg/helper.py"]);
        let offenders = scan(tree.path(), &["*_test.py".to_string()]).unwrap();
        assert!(offenders.is_empty());
    }

    #[test]
    fn scan_matches_any_of_several_globs_and_returns_sorted() {
        let tree = TempTree::new(&["a.test.ts", "b_test.py", "keep.ts"]);
        let globs = vec!["*_test.py".to_string(), "*.test.*".to_string()];
        let offenders = scan(tree.path(), &globs).unwrap();
        assert_eq!(
            offenders,
            vec![tree.path().join("a.test.ts"), tree.path().join("b_test.py")],
        );
    }

    #[test]
    fn scan_errors_when_the_root_cannot_be_read() {
        let missing = std::env::temp_dir().join("tc-packaging-does-not-exist-9f8e7d");
        assert!(scan(&missing, &["*_test.py".to_string()]).is_err());
    }
}
