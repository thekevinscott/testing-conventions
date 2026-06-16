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

use anyhow::{bail, Context, Result};

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
    collect_offenders(root, root, globs, &mut offenders)?;
    offenders.sort();
    Ok(offenders)
}

/// Inspect a built artifact at `path` for files matching `globs` — the test-file
/// patterns that must not ship.
///
/// `path` is either a **directory** (an already-unpacked artifact) or a packed
/// archive this rule understands — a Python wheel (`.whl`, a zip) or a gzipped tar
/// (`.tgz` / `.tar.gz`, e.g. an `npm pack` tarball or Python sdist; a Cargo
/// `.crate` too) — which is unpacked into a scratch directory first. Either way
/// the unpacked tree is handed to [`scan`]. Offenders come back as paths
/// **relative to the artifact root** (e.g. `package/dist/widget.test.js`), so they
/// read the same whether the artifact was a directory or an archive. Errors if the
/// artifact can't be read, or isn't a directory or a recognized archive.
pub fn inspect(path: impl AsRef<Path>, globs: &[String]) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(relative_to(path, scan(path, globs)?));
    }
    let unpacked = if is_zip_artifact(path) {
        unzip_to_temp(path)?
    } else if is_tar_gz_artifact(path) {
        untar_gz_to_temp(path)?
    } else {
        bail!(
            "`{}` is not a directory or a recognized built artifact \
             (expected a directory, a `.whl`, a `.tgz`/`.tar.gz`, or a `.crate`)",
            path.display()
        )
    };
    Ok(relative_to(unpacked.path(), scan(unpacked.path(), globs)?))
}

/// `true` for an artifact this rule unpacks as a zip: a Python wheel (`.whl`) or
/// a plain `.zip`.
fn is_zip_artifact(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("whl" | "zip")
    )
}

/// Re-express each offender as a path relative to `root`. [`scan`] returns paths
/// under `root`, so the strip always succeeds; an unexpected path is kept as-is.
fn relative_to(root: &Path, offenders: Vec<PathBuf>) -> Vec<PathBuf> {
    offenders
        .into_iter()
        .map(|p| p.strip_prefix(root).map(Path::to_path_buf).unwrap_or(p))
        .collect()
}

/// Unpack a zip artifact into a fresh scratch directory (removed on drop).
fn unzip_to_temp(archive: &Path) -> Result<TempDir> {
    let file = std::fs::File::open(archive)
        .with_context(|| format!("opening artifact `{}`", archive.display()))?;
    let mut zip = zip::ZipArchive::new(file)
        .with_context(|| format!("reading `{}` as a zip archive", archive.display()))?;
    let dir = TempDir::new()?;
    zip.extract(dir.path())
        .with_context(|| format!("unpacking `{}`", archive.display()))?;
    Ok(dir)
}

/// `true` for an artifact this rule unpacks as a gzipped tar: an `npm pack`
/// tarball (`.tgz`), a `.tar.gz` (a Python sdist), or a Cargo `.crate` from
/// `cargo package` (#74) — all gzipped tarballs.
fn is_tar_gz_artifact(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name.ends_with(".tgz") || name.ends_with(".tar.gz") || name.ends_with(".crate")
}

/// Unpack a gzipped-tar artifact into a fresh scratch directory (removed on drop).
fn untar_gz_to_temp(archive: &Path) -> Result<TempDir> {
    let file = std::fs::File::open(archive)
        .with_context(|| format!("opening artifact `{}`", archive.display()))?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let dir = TempDir::new()?;
    tar.unpack(dir.path())
        .with_context(|| format!("unpacking `{}`", archive.display()))?;
    Ok(dir)
}

/// A scratch directory removed on drop — where an archive artifact is unpacked.
/// Unique per call (so parallel checks don't collide) and cleaned up so nothing
/// leaks into the temp dir.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "testing-conventions-pkg-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&path)
            .with_context(|| format!("creating scratch directory `{}`", path.display()))?;
        Ok(TempDir(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Recursively collect every file under `dir` (within the artifact `root`) that
/// matches one of `patterns`.
fn collect_offenders(
    dir: &Path,
    root: &Path,
    patterns: &[String],
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_offenders(&path, root, patterns, out)?;
        } else if matches_any(&path, root, patterns) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` when `path` matches any of `patterns`.
///
/// A pattern ending in `/` is a **directory** pattern: it matches when `path`
/// (relative to the artifact `root`) lives under a directory of that name — e.g.
/// `tests/` flags `…/tests/integration.rs` (Rust's crate-root integration tests,
/// #74). Every other pattern is a file-name glob (`*` wildcards) matched against
/// the entry's name (`*_test.py`, `*.test.*`).
fn matches_any(path: &Path, root: &Path, patterns: &[String]) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    patterns
        .iter()
        .any(|pattern| match pattern.strip_suffix('/') {
            Some(dir) => path_under_dir(path, root, dir),
            None => matches_glob(pattern, name),
        })
}

/// `true` when `path` (relative to `root`) has an **ancestor** directory named
/// `dir` — i.e. the file lives somewhere under a `dir/`.
fn path_under_dir(path: &Path, root: &Path, dir: &str) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .parent()
        .is_some_and(|parents| parents.components().any(|c| c.as_os_str() == dir))
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
    fn a_directory_pattern_flags_files_under_that_dir() {
        let tree = TempTree::new(&["tests/integration.rs", "src/lib.rs", "src/tests/nested.rs"]);
        let offenders = scan(tree.path(), &["tests/".to_string()]).unwrap();
        // Any file with a `tests/` ancestor is flagged (here the crate-root
        // `tests/` and a nested `src/tests/`); `src/lib.rs` is not.
        assert_eq!(
            offenders,
            vec![
                tree.path().join("src/tests/nested.rs"),
                tree.path().join("tests/integration.rs"),
            ],
        );
    }

    #[test]
    fn recognizes_a_dot_crate_as_a_gzipped_tar() {
        assert!(is_tar_gz_artifact(Path::new("widget-0.1.0.crate")));
        assert!(is_tar_gz_artifact(Path::new("pkg.tgz")));
        assert!(is_tar_gz_artifact(Path::new("pkg.tar.gz")));
        assert!(!is_tar_gz_artifact(Path::new("pkg.whl")));
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

    #[test]
    fn inspect_scans_a_directory_artifact_with_relative_paths() {
        let tree = TempTree::new(&["pkg/widget.py", "pkg/widget_test.py"]);
        let offenders = inspect(tree.path(), &["*_test.py".to_string()]).unwrap();
        assert_eq!(offenders, vec![PathBuf::from("pkg/widget_test.py")]);
    }

    #[test]
    fn inspect_rejects_an_unrecognized_artifact() {
        let tree = TempTree::new(&["not-an-archive.txt"]);
        let err = inspect(
            tree.path().join("not-an-archive.txt"),
            &["*_test.py".to_string()],
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("not a directory or a recognized"),
            "got: {err}"
        );
    }
}
