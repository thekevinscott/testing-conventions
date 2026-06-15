//! Unit-test location/naming check (Python — issue #15; TypeScript — issue #18).
//!
//! Convention (README "Location & Naming"; `internals/*/testing.md`): a source
//! file is unit-tested by a *colocated* test named after it — `foo.py` →
//! `foo_test.py` (Python), `foo-bar.ts` → `foo-bar.test.ts` (TypeScript).
//! [`missing_unit_tests`] walks a tree for a [`Language`] and returns every
//! source file with no such sibling — an "orphan". Test files are what the
//! check looks *for*, never subjects.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// A language whose unit-test location/naming convention can be checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Language {
    /// `foo.py` → colocated `foo_test.py`; `__init__.py` is exempt.
    #[value(name = "python")]
    Python,
    /// `foo-bar.ts` → colocated `foo-bar.test.ts`; `*.d.ts` is ignored.
    #[value(name = "typescript")]
    TypeScript,
}

impl Language {
    /// `true` for a file this language's check tracks (source *or* test).
    fn tracks(self, path: &Path) -> bool {
        match self {
            Language::Python => has_extension(path, &["py"]),
            Language::TypeScript => has_extension(path, &["ts", "tsx"]) && !is_declaration(path),
        }
    }

    /// `true` when `path` is itself a unit test, never a subject.
    fn is_test(self, path: &Path) -> bool {
        match self {
            Language::Python => stem_of(path).ends_with("_test"),
            Language::TypeScript => {
                let name = file_name_of(path);
                name.ends_with(".test.ts") || name.ends_with(".test.tsx")
            }
        }
    }

    /// `true` for a file exempt from needing a colocated test.
    fn is_exempt(self, path: &Path) -> bool {
        match self {
            Language::Python => file_name_of(path) == "__init__.py",
            Language::TypeScript => false,
        }
    }

    /// The colocated test `source` is expected to have.
    fn expected_test_path(self, source: &Path) -> PathBuf {
        match self {
            Language::Python => source.with_file_name(format!("{}_test.py", stem_of(source))),
            Language::TypeScript => {
                source.with_file_name(format!("{}.test.{}", stem_of(source), extension_of(source)))
            }
        }
    }
}

/// Walk `root` recursively and return every source file (for `language`) that
/// has no colocated unit test, sorted for deterministic output.
///
/// A file that is itself a test is never treated as a subject; every other
/// source file must have its colocated test sibling. Returns an error if the
/// tree under `root` cannot be read.
pub fn missing_unit_tests(root: impl AsRef<Path>, language: Language) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(root.as_ref(), language, &mut files)?;

    // Every tracked path we found, so a subject's expected twin is a lookup
    // rather than a second pass over the filesystem.
    let present: HashSet<&Path> = files.iter().map(PathBuf::as_path).collect();

    let mut orphans: Vec<PathBuf> = Vec::new();
    for source in &files {
        if language.is_test(source) || language.is_exempt(source) {
            continue;
        }
        if !present.contains(language.expected_test_path(source).as_path()) {
            orphans.push(source.clone());
        }
    }
    orphans.sort();
    Ok(orphans)
}

/// Recursively collect every file `language` tracks under `dir` into `out`.
fn collect_files(dir: &Path, language: Language, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_files(&path, language, out)?;
        } else if language.tracks(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` when the file's extension is one of `extensions`.
fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| extensions.contains(&ext))
}

/// `true` for a TypeScript declaration file (`*.d.ts`) — no runtime code, so
/// never a unit-test subject.
fn is_declaration(path: &Path) -> bool {
    file_name_of(path).ends_with(".d.ts")
}

/// The file extension, lossily decoded (empty if there is none).
fn extension_of(path: &Path) -> String {
    path.extension()
        .map(|ext| ext.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// The file name, lossily decoded.
fn file_name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
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
    fn python_tracks_py_files() {
        assert!(Language::Python.tracks(Path::new("a.py")));
        assert!(Language::Python.tracks(Path::new("pkg/widget.py")));
        assert!(!Language::Python.tracks(Path::new("a.pyi")));
        assert!(!Language::Python.tracks(Path::new("a.txt")));
        assert!(!Language::Python.tracks(Path::new("README")));
    }

    #[test]
    fn python_recognizes_test_files_by_stem_suffix() {
        assert!(Language::Python.is_test(Path::new("widget_test.py")));
        assert!(Language::Python.is_test(Path::new("pkg/helper_test.py")));
        assert!(!Language::Python.is_test(Path::new("widget.py")));
    }

    #[test]
    fn python_exempts_the_package_marker() {
        assert!(Language::Python.is_exempt(Path::new("__init__.py")));
        assert!(Language::Python.is_exempt(Path::new("pkg/__init__.py")));
        assert!(!Language::Python.is_exempt(Path::new("conftest.py")));
        assert!(!Language::Python.is_exempt(Path::new("widget.py")));
    }

    #[test]
    fn python_expected_test_path_is_the_colocated_twin() {
        assert_eq!(
            Language::Python.expected_test_path(Path::new("pkg/widget.py")),
            PathBuf::from("pkg/widget_test.py")
        );
        assert_eq!(
            Language::Python.expected_test_path(Path::new("widget.py")),
            PathBuf::from("widget_test.py")
        );
    }

    #[test]
    fn typescript_tracks_ts_tsx_mts_cts_but_not_declarations() {
        assert!(Language::TypeScript.tracks(Path::new("widget.ts")));
        assert!(Language::TypeScript.tracks(Path::new("pkg/button.tsx")));
        assert!(Language::TypeScript.tracks(Path::new("service.mts")));
        assert!(Language::TypeScript.tracks(Path::new("legacy.cts")));
        assert!(!Language::TypeScript.tracks(Path::new("types.d.ts")));
        assert!(!Language::TypeScript.tracks(Path::new("ambient.d.mts")));
        assert!(!Language::TypeScript.tracks(Path::new("globals.d.cts")));
        assert!(!Language::TypeScript.tracks(Path::new("widget.py")));
        assert!(!Language::TypeScript.tracks(Path::new("README")));
    }

    #[test]
    fn typescript_recognizes_test_files_by_suffix() {
        assert!(Language::TypeScript.is_test(Path::new("widget.test.ts")));
        assert!(Language::TypeScript.is_test(Path::new("pkg/button.test.tsx")));
        assert!(Language::TypeScript.is_test(Path::new("service.test.mts")));
        assert!(Language::TypeScript.is_test(Path::new("legacy.test.cts")));
        assert!(!Language::TypeScript.is_test(Path::new("widget.ts")));
        assert!(!Language::TypeScript.is_test(Path::new("button.tsx")));
        assert!(!Language::TypeScript.is_test(Path::new("service.mts")));
    }

    #[test]
    fn typescript_has_no_exemptions() {
        // Unlike Python's `__init__.py`, nothing in TS is language-mandated;
        // `index.ts` is a deliberate file and therefore a subject.
        assert!(!Language::TypeScript.is_exempt(Path::new("index.ts")));
        assert!(!Language::TypeScript.is_exempt(Path::new("pkg/index.ts")));
    }

    #[test]
    fn typescript_expected_test_path_keeps_the_extension() {
        assert_eq!(
            Language::TypeScript.expected_test_path(Path::new("pkg/widget.ts")),
            PathBuf::from("pkg/widget.test.ts")
        );
        assert_eq!(
            Language::TypeScript.expected_test_path(Path::new("button.tsx")),
            PathBuf::from("button.test.tsx")
        );
        assert_eq!(
            Language::TypeScript.expected_test_path(Path::new("service.mts")),
            PathBuf::from("service.test.mts")
        );
        assert_eq!(
            Language::TypeScript.expected_test_path(Path::new("legacy.cts")),
            PathBuf::from("legacy.test.cts")
        );
    }
}
