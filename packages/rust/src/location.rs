//! Unit-test location/naming check (Python — issue #15; TypeScript — issue #18;
//! exemptions & waivers — issue #32).
//!
//! Convention (README "Location & Naming"; `internals/*/testing.md`): a source
//! file is unit-tested by a *colocated* test named after it — `foo.py` →
//! `foo_test.py` (Python), `foo-bar.ts` → `foo-bar.test.ts` (TypeScript).
//! [`missing_unit_tests`] walks a tree for a [`Language`] and returns every
//! source file with no such sibling — an "orphan". Test files are what the
//! check looks *for*, never subjects.
//!
//! Three things are *not* orphans even without a colocated test (issue #32):
//! language-mandated markers ([`Language::is_exempt`] — `__init__.py`), pure
//! re-export **barrels** matched by shape ([`Language::is_barrel`] — the
//! TypeScript analog of `__init__.py`), and files carrying an explicit,
//! reason-required [`waiver`](crate::waiver). Everything else must be tested.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::waiver::{self, Scope};

/// A language whose unit-test location/naming convention can be checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Language {
    /// `foo.py` → colocated `foo_test.py`; `__init__.py` is exempt.
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

    /// `true` for a file exempt *by name* from needing a colocated test — a
    /// language-mandated marker that the language forces into existence and that
    /// carries no testable logic. Cheap (path only); see [`Self::is_barrel`] for
    /// the by-shape exemption.
    fn is_exempt(self, path: &Path) -> bool {
        match self {
            Language::Python => file_name_of(path) == "__init__.py",
            Language::TypeScript => false,
        }
    }

    /// `true` when `source` (the file's contents) is exempt *by shape* — a pure
    /// re-export barrel whose only statements are `export … from "…"`
    /// re-exports. This is the TypeScript analog of Python's `__init__.py`
    /// (issue #32): a barrel wires modules together but holds no runtime logic
    /// of its own, so there is nothing to unit-test. Matched by shape, not name,
    /// so `index.ts`, `public-api.ts`, and the like are all covered. Python's
    /// re-exports live in the already-exempt `__init__.py`, so this is always
    /// `false` there.
    fn is_barrel(self, source: &str) -> bool {
        match self {
            Language::Python => false,
            Language::TypeScript => is_reexport_barrel(source),
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
/// source file must have its colocated test sibling — unless it is exempt by
/// name (`__init__.py`), exempt by shape (a re-export barrel), or carries a
/// `location`/`all` [`waiver`](crate::waiver). A malformed waiver (no reason,
/// unknown scope) is an error, never a silent pass. Returns an error if the tree
/// under `root` cannot be read.
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
        if present.contains(language.expected_test_path(source).as_path()) {
            continue;
        }
        // No colocated test. A by-shape barrel or a reason-required waiver can
        // still exempt it; anything else is an orphan. Both need the contents,
        // read only now — for the handful of files that lack a twin.
        let contents = std::fs::read_to_string(source)
            .with_context(|| format!("reading source file `{}`", source.display()))?;
        if language.is_barrel(&contents) {
            continue;
        }
        let waived = waiver::waived_reason(&contents, Scope::Location)
            .with_context(|| format!("checking waivers in `{}`", source.display()))?;
        if waived.is_none() {
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

/// `true` for a TypeScript declaration file (`*.d.ts` / `*.d.mts` / `*.d.cts`) —
/// no runtime code, so never a unit-test subject.
fn is_declaration(path: &Path) -> bool {
    let name = file_name_of(path);
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
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

/// `true` when `source` is a pure re-export barrel: its only statements are
/// `export … from "…"` re-exports. Comments and blank lines are ignored; a file
/// with any other statement (a local declaration, an `import`, side-effecting
/// code), or with no re-export at all, is not a barrel. Recognized in
/// conventionally-formatted source (statements terminated by `;` or a newline).
fn is_reexport_barrel(source: &str) -> bool {
    let statements = top_level_statements(source);
    !statements.is_empty() && statements.iter().all(|statement| is_reexport(statement))
}

/// `true` when `statement` (whitespace already collapsed and trimmed) is a
/// single `export … from "…"` re-export — `export * from`, `export * as ns
/// from`, `export { … } from`, or any of those with a `type` modifier. The
/// leading shape plus the ` from ` specifier together exclude local exports
/// (`export const x = …`, `export function …`), which carry testable logic.
fn is_reexport(statement: &str) -> bool {
    let shape = statement.starts_with("export * ")
        || statement.starts_with("export {")
        || statement.starts_with("export type {")
        || statement.starts_with("export type * ");
    shape && statement.contains(" from ")
}

/// Split TypeScript `source` into top-level statements, skipping comments and
/// respecting string literals so a delimiter inside one doesn't count. A
/// statement breaks on a top-level `;` or a newline at brace depth 0 (TypeScript
/// inserts the semicolon); a `{ … }` keeps its interior newlines from splitting
/// a multi-line export list. Each statement has its whitespace collapsed to
/// single spaces and is trimmed; empty statements are dropped.
fn top_level_statements(source: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut depth: i32 = 0;
    let mut chars = source.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Line comment: drop its body, but leave the terminating newline for
            // the `\n` arm — so a comment inside `{ … }` can't break a statement.
            '/' if chars.peek() == Some(&'/') => {
                while chars.peek().is_some_and(|&n| n != '\n') {
                    chars.next();
                }
            }
            // Block comment: drop everything through the closing `*/`.
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
            // String literal: copy verbatim, honoring escapes, so `;`/`from`/a
            // quote inside it is never treated as syntax.
            '\'' | '"' | '`' => {
                current.push(c);
                while let Some(n) = chars.next() {
                    current.push(n);
                    if n == '\\' {
                        if let Some(escaped) = chars.next() {
                            current.push(escaped);
                        }
                    } else if n == c {
                        break;
                    }
                }
            }
            '{' => {
                depth += 1;
                current.push(c);
            }
            '}' => {
                depth -= 1;
                current.push(c);
            }
            ';' if depth == 0 => push_statement(&mut statements, &mut current),
            '\n' if depth == 0 => push_statement(&mut statements, &mut current),
            _ => current.push(c),
        }
    }
    // Flush any trailing statement (e.g. a final re-export with no semicolon, or
    // unterminated leftover — which won't match a re-export and so isn't a barrel).
    push_statement(&mut statements, &mut current);
    statements
}

/// Collapse `current`'s whitespace and, if anything remains, push it as a
/// statement; then clear `current`.
fn push_statement(statements: &mut Vec<String>, current: &mut String) {
    let collapsed = current.split_whitespace().collect::<Vec<_>>().join(" ");
    current.clear();
    if !collapsed.is_empty() {
        statements.push(collapsed);
    }
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
    fn typescript_has_no_by_name_exemptions() {
        // Nothing in TS is exempt by *name* — unlike Python's language-mandated
        // `__init__.py`. `index.ts` earns its exemption by *shape* (see the
        // barrel tests), not because of its name, so the path-only check is
        // always false here.
        assert!(!Language::TypeScript.is_exempt(Path::new("index.ts")));
        assert!(!Language::TypeScript.is_exempt(Path::new("pkg/index.ts")));
    }

    #[test]
    fn a_pure_reexport_file_is_a_barrel() {
        let barrel = "\
export * from './widget';
export { Button } from './button';
export type { Props } from './button';
export * as utils from './utils';
";
        assert!(Language::TypeScript.is_barrel(barrel));
        // Matched by shape, not name — Python never treats a file as a barrel.
        assert!(!Language::Python.is_barrel(barrel));
    }

    #[test]
    fn a_multiline_reexport_list_is_still_a_barrel() {
        let barrel = "\
export {
  alpha,
  beta,
} from './letters';
";
        assert!(Language::TypeScript.is_barrel(barrel));
    }

    #[test]
    fn comments_and_blank_lines_do_not_disqualify_a_barrel() {
        let barrel = "\
/* SPDX-License-Identifier: MIT */
// public surface

export * from './a'; // re-export everything
export { b } from './b';
";
        assert!(Language::TypeScript.is_barrel(barrel));
    }

    #[test]
    fn a_file_with_runtime_logic_is_not_a_barrel() {
        // A local declaration alongside re-exports carries testable logic.
        assert!(!Language::TypeScript
            .is_barrel("export * from './a';\nexport const VERSION = '1.0.0';\n"));
        assert!(!Language::TypeScript.is_barrel("export function greet() {\n  return 'hi';\n}\n"));
    }

    #[test]
    fn import_then_export_is_not_a_barrel() {
        // The rule is strictly "only does `export … from`"; an import plus a bare
        // `export { … }` is not the shape we exempt.
        assert!(!Language::TypeScript
            .is_barrel("import { thing } from './thing';\nexport { thing };\n"));
    }

    #[test]
    fn an_empty_or_comment_only_file_is_not_a_barrel() {
        assert!(!Language::TypeScript.is_barrel(""));
        assert!(!Language::TypeScript.is_barrel("// nothing here\n"));
    }

    #[test]
    fn a_from_inside_a_string_does_not_make_a_barrel() {
        // The ` from ` specifier must be syntax, not text inside a module string.
        assert!(!Language::TypeScript.is_barrel("export const note = 'export x from y';\n"));
    }

    #[test]
    fn an_escaped_quote_does_not_end_a_string_early() {
        // `'a\'b from c'` is one string literal; the file carries runtime logic
        // (not a barrel), and the escaped quote must not read as the closing one.
        assert!(!Language::TypeScript.is_barrel("export const path = 'a\\'b from c';\n"));
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
