//! Unit-test location/naming check for Python sources (issue #15).
//!
//! The convention (README "Location & Naming"; `internals/python/testing.md`):
//! a Python source file `foo.py` is unit-tested by a colocated `foo_test.py`.
//! [`missing_unit_tests`] walks a directory tree and returns every source file
//! that has no such sibling — an "orphan". Files that are themselves tests
//! (`*_test.py`) are what the check looks *for*, never subjects, and the
//! package marker (`__init__.py`) is exempt.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// The extension that marks a Python file.
const PY_EXTENSION: &str = "py";
/// The stem suffix that marks a file as a unit test: `foo` → `foo_test`.
const TEST_STEM_SUFFIX: &str = "_test";
/// The package marker, which is never a unit-test subject.
const PACKAGE_MARKER: &str = "__init__.py";

/// Walk `root` recursively and return every Python source file that has no
/// colocated `<stem>_test.py`, sorted for deterministic output.
///
/// A file whose stem ends in `_test` is itself a test and is never treated as a
/// subject; every other `*.py` file is a subject and must have its colocated
/// test sibling. Returns an error if the tree under `root` cannot be read.
pub fn missing_unit_tests(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut python_files = Vec::new();
    collect_python_files(root.as_ref(), &mut python_files)?;

    // Every `*.py` path we found, so a subject's expected twin is a lookup
    // rather than a second pass over the filesystem.
    let present: HashSet<&Path> = python_files.iter().map(PathBuf::as_path).collect();

    let mut orphans: Vec<PathBuf> = Vec::new();
    for source in &python_files {
        if is_test_file(source) || is_exempt(source) {
            continue;
        }
        if !present.contains(expected_test_path(source).as_path()) {
            orphans.push(source.clone());
        }
    }
    orphans.sort();
    Ok(orphans)
}

/// Recursively collect every `*.py` file under `dir` into `out`.
fn collect_python_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_python_files(&path, out)?;
        } else if is_python_source(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` for a file with a `.py` extension.
fn is_python_source(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some(PY_EXTENSION)
}

/// `true` when `path` is itself a unit test (`*_test.py`), never a subject.
fn is_test_file(path: &Path) -> bool {
    stem_of(path).ends_with(TEST_STEM_SUFFIX)
}

/// `true` for the package marker (`__init__.py`), which never needs a test.
fn is_exempt(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some(PACKAGE_MARKER)
}

/// The colocated test a source is expected to have: `foo.py` → `foo_test.py`.
fn expected_test_path(source: &Path) -> PathBuf {
    source.with_file_name(format!(
        "{}{}.{}",
        stem_of(source),
        TEST_STEM_SUFFIX,
        PY_EXTENSION
    ))
}

/// The file stem (the name without its extension), lossily decoded.
fn stem_of(path: &Path) -> String {
    path.file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_python_sources_by_extension() {
        assert!(is_python_source(Path::new("a.py")));
        assert!(is_python_source(Path::new("pkg/widget.py")));
        assert!(!is_python_source(Path::new("a.pyi")));
        assert!(!is_python_source(Path::new("a.txt")));
        assert!(!is_python_source(Path::new("README")));
    }

    #[test]
    fn recognizes_test_files_by_stem_suffix() {
        assert!(is_test_file(Path::new("widget_test.py")));
        assert!(is_test_file(Path::new("pkg/helper_test.py")));
        assert!(!is_test_file(Path::new("widget.py")));
        assert!(!is_test_file(Path::new("pkg/helper.py")));
    }

    #[test]
    fn exempts_the_package_marker() {
        assert!(is_exempt(Path::new("__init__.py")));
        assert!(is_exempt(Path::new("pkg/__init__.py")));
        assert!(!is_exempt(Path::new("conftest.py")));
        assert!(!is_exempt(Path::new("widget.py")));
    }

    #[test]
    fn expected_test_path_is_the_colocated_twin() {
        assert_eq!(
            expected_test_path(Path::new("pkg/widget.py")),
            PathBuf::from("pkg/widget_test.py")
        );
        assert_eq!(
            expected_test_path(Path::new("widget.py")),
            PathBuf::from("widget_test.py")
        );
    }
}
