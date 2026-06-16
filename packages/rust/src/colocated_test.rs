//! The unit `colocated-test` check (Python — issue #15; TypeScript — issue #18;
//! exemptions — issue #32).
//!
//! Convention (README "Colocated Test"; `internals/*/testing.md`): a source
//! file is unit-tested by a *colocated* test named after it — `foo.py` →
//! `foo_test.py` (Python), `foo-bar.ts` → `foo-bar.test.ts` (TypeScript).
//! [`missing_unit_tests`] walks a tree for a [`Language`] and returns every
//! source file with no such sibling — an "orphan". Test files are what the
//! check looks *for*, never subjects.
//!
//! Two things are not orphans even without a colocated test (issue #32): a file
//! that holds no code (empty or comment-only — e.g. a bare `__init__.py`), which
//! is not a subject at all, and a file listed in the config `exempt` table,
//! which is a deliberate, reason-required omission. Everything else must be
//! tested — there is no automatic name- or shape-based exemption.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// A language whose colocated unit-test convention can be checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Language {
    /// `foo.py` → colocated `foo_test.py`.
    #[value(name = "python")]
    Python,
    /// `foo-bar.ts` → colocated `foo-bar.test.ts`, across `.ts`/`.tsx`/`.mts`/`.cts`;
    /// declaration files (`.d.ts`/`.d.mts`/`.d.cts`) are ignored.
    #[value(name = "typescript")]
    TypeScript,
}

impl Language {
    /// `true` for a file this language's check tracks (source *or* test).
    fn tracks(self, path: &Path) -> bool {
        match self {
            Language::Python => has_extension(path, &["py"]),
            Language::TypeScript => {
                has_extension(path, &["ts", "tsx", "mts", "cts"]) && !is_declaration(path)
            }
        }
    }

    /// `true` when `path` is itself a unit test, never a subject.
    fn is_test(self, path: &Path) -> bool {
        match self {
            Language::Python => stem_of(path).ends_with("_test"),
            Language::TypeScript => {
                let name = file_name_of(path);
                name.ends_with(".test.ts")
                    || name.ends_with(".test.tsx")
                    || name.ends_with(".test.mts")
                    || name.ends_with(".test.cts")
            }
        }
    }

    /// `true` when `source` (the file's contents) holds at least one line of
    /// code — anything beyond blank lines and comments. An empty or comment-only
    /// file (e.g. a bare `__init__.py`) carries no logic, so it is never a
    /// unit-test subject and needs no exemption (issue #32).
    fn has_code(self, source: &str) -> bool {
        match self {
            Language::Python => python_has_code(source),
            Language::TypeScript => typescript_has_code(source),
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
/// A file that is itself a test is never a subject; an empty/comment-only file
/// holds no logic and is never a subject; a file whose `root`-relative path is
/// in `exempt` is a deliberate, reason-required omission. Every other source
/// file must have its colocated test sibling. `exempt` holds the
/// `colocated-test`-rule paths resolved from config
/// ([`crate::config::resolve_exempt`]). Returns an
/// error if the tree under `root` cannot be read.
pub fn missing_unit_tests(
    root: impl AsRef<Path>,
    language: Language,
    exempt: &BTreeSet<String>,
) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_files(root, language, &mut files)?;

    // Every tracked path we found, so a subject's expected twin is a lookup
    // rather than a second pass over the filesystem.
    let present: HashSet<&Path> = files.iter().map(PathBuf::as_path).collect();

    let mut orphans: Vec<PathBuf> = Vec::new();
    for source in &files {
        if language.is_test(source) {
            continue;
        }
        if present.contains(language.expected_test_path(source).as_path()) {
            continue;
        }
        // No colocated test. An empty/comment-only file is not a subject; read
        // only now — for the handful of files that lack a twin — to find out.
        let contents = std::fs::read_to_string(source)
            .with_context(|| format!("reading source file `{}`", source.display()))?;
        if !language.has_code(&contents) {
            continue;
        }
        let relative = source
            .strip_prefix(root)
            .unwrap_or(source)
            .to_string_lossy()
            .replace('\\', "/");
        if exempt.contains(&relative) {
            continue;
        }
        orphans.push(source.clone());
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

/// `true` for a TypeScript declaration file (`*.d.ts` / `*.d.mts` / `*.d.cts`) —
/// no runtime code, so never a unit-test subject.
fn is_declaration(path: &Path) -> bool {
    let name = file_name_of(path);
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}

/// `true` when any line of Python `source` is neither blank nor a `#` comment. A
/// module docstring counts as code (it is non-comment content).
fn python_has_code(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    })
}

/// `true` when TypeScript `source` holds anything beyond whitespace and comments
/// (`//` line, `/* … */` block). Any other character — including the start of a
/// string literal — counts as code.
fn typescript_has_code(source: &str) -> bool {
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            c if c.is_whitespace() => {}
            '/' if chars.peek() == Some(&'/') => {
                while chars.peek().is_some_and(|&n| n != '\n') {
                    chars.next();
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = '\0';
                for n in chars.by_ref() {
                    if prev == '*' && n == '/' {
                        break;
                    }
                    prev = n;
                }
            }
            _ => return true,
        }
    }
    false
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

    #[test]
    fn python_empty_or_comment_only_files_have_no_code() {
        assert!(!Language::Python.has_code(""));
        assert!(!Language::Python.has_code("\n   \n"));
        assert!(!Language::Python.has_code("# just a comment\n   # another\n"));
    }

    #[test]
    fn python_real_content_counts_as_code() {
        assert!(Language::Python.has_code("x = 1\n"));
        assert!(Language::Python.has_code("# header\nimport os\n"));
        // A docstring is non-comment content, so it counts.
        assert!(Language::Python.has_code("\"\"\"Package docstring.\"\"\"\n"));
    }

    #[test]
    fn typescript_empty_or_comment_only_files_have_no_code() {
        assert!(!Language::TypeScript.has_code(""));
        assert!(!Language::TypeScript.has_code("   \n\t\n"));
        assert!(!Language::TypeScript.has_code("// a line comment\n"));
        assert!(!Language::TypeScript.has_code("/* a\n   block\n   comment */\n"));
    }

    #[test]
    fn typescript_real_content_counts_as_code() {
        assert!(Language::TypeScript.has_code("export const x = 1;\n"));
        assert!(Language::TypeScript.has_code("// note\nexport * from './a';\n"));
        // A string literal (even one that looks comment-ish) is code.
        assert!(Language::TypeScript.has_code("const s = '// not a comment';\n"));
        // A lone division slash is code, not a comment.
        assert!(Language::TypeScript.has_code("const r = a / b;\n"));
    }
}
