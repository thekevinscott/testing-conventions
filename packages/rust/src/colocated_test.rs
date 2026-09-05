//! The unit `colocated-test` check.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rustpython_parser::lexer::lex;
use rustpython_parser::{ast, Mode, Parse, Tok};
use syn::visit::{self, Visit};

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
    /// Rust units are inline `#[cfg(test)]` modules, not separate files, so the
    /// file-pairing walk below does not apply to Rust; its arm of the rule checks
    /// inline-`#[cfg(test)]` *presence* instead ([`missing_inline_tests`]). The
    /// variant is also accepted by the other `--language` rules (e.g. `packaging`).
    #[value(name = "rust")]
    Rust,
}

impl Language {
    /// `true` for a file this language's check tracks (source *or* test).
    pub(crate) fn tracks(self, path: &Path) -> bool {
        match self {
            Language::Python => has_extension(path, &["py"]),
            Language::TypeScript => {
                has_extension(path, &["ts", "tsx", "mts", "cts"]) && !is_declaration(path)
            }
            Language::Rust => false,
        }
    }

    /// `true` when `path` is itself a unit test, never a subject.
    pub(crate) fn is_test(self, path: &Path) -> bool {
        match self {
            Language::Python => stem_of(path).ends_with("_test"),
            Language::TypeScript => {
                let name = file_name_of(path);
                name.ends_with(".test.ts")
                    || name.ends_with(".test.tsx")
                    || name.ends_with(".test.mts")
                    || name.ends_with(".test.cts")
            }
            Language::Rust => false,
        }
    }

    /// `true` when `path` is test *support* — Python's `conftest.py`, never a subject.
    pub(crate) fn is_support(self, path: &Path) -> bool {
        match self {
            Language::Python => file_name_of(path) == "conftest.py",
            Language::TypeScript | Language::Rust => false,
        }
    }

    /// `true` when `source` holds at least one line of code — anything beyond blank
    /// lines and comments.
    pub(crate) fn has_code(self, source: &str) -> bool {
        match self {
            Language::Python => python_has_code(source),
            Language::TypeScript => typescript_has_code(source),
            Language::Rust => false,
        }
    }

    /// `true` when `source` at `path` declares behavior a unit test can exercise. Presence
    /// and the commit-scoped co-change check both decide subjecthood here, so they cannot
    /// disagree about what has behavior.
    pub(crate) fn is_subject(self, source: &str, path: &Path) -> bool {
        if !self.has_code(source) {
            return false;
        }
        match self {
            Language::TypeScript => !crate::ts::is_type_only_module(source, path),
            Language::Python | Language::Rust => true,
        }
    }

    /// `true` when `base` and `head` — the file at `path` before and after an edit — hold
    /// the same code once comments and formatting whitespace are normalized away. Content
    /// that fails to parse on either side is **not** equal.
    pub(crate) fn same_code(self, base: &str, head: &str, path: &Path) -> bool {
        match self {
            Language::Python => python_same_code(base, head),
            Language::TypeScript => crate::ts::same_code(base, head, path),
            // Unreachable for Rust; `false` keeps any caller that arrives flagged.
            Language::Rust => false,
        }
    }

    /// The colocated test `source` is expected to have.
    pub(crate) fn expected_test_path(self, source: &Path) -> PathBuf {
        match self {
            Language::Python => source.with_file_name(format!("{}_test.py", stem_of(source))),
            Language::TypeScript => {
                source.with_file_name(format!("{}.test.{}", stem_of(source), extension_of(source)))
            }
            // Unreachable for Rust (nothing is tracked); a harmless identity.
            Language::Rust => source.to_path_buf(),
        }
    }
}

/// Every source file under `root` (for `language`) with no colocated unit test, sorted.
/// `exempt` holds the rule's `root`-relative paths resolved from config
/// ([`crate::config::resolve_exempt`]).
pub fn missing_unit_tests(
    root: impl AsRef<Path>,
    language: Language,
    exempt: &BTreeSet<String>,
) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_files(root, language, &mut files)?;
    // `<package root>/tests/` belongs to the suite tiers, so nothing under it is a subject.
    let manifest = match language {
        Language::Python => Some("pyproject.toml"),
        Language::TypeScript => Some("package.json"),
        Language::Rust => None,
    };
    if let Some(tests) = manifest.and_then(|m| crate::tiers::suite_tests_dir(root, m)) {
        files.retain(|file| !file.starts_with(&tests));
    }

    let present: HashSet<&Path> = files.iter().map(PathBuf::as_path).collect();

    let mut orphans: Vec<PathBuf> = Vec::new();
    for source in &files {
        if language.is_test(source) || language.is_support(source) {
            continue;
        }
        if present.contains(language.expected_test_path(source).as_path()) {
            continue;
        }
        // Read only for a file that lacks a twin, so the common case costs no file read.
        let contents = std::fs::read_to_string(source)
            .with_context(|| format!("reading source file `{}`", source.display()))?;
        if !language.is_subject(&contents, source) {
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
pub(crate) fn collect_files(dir: &Path, language: Language, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = crate::walk::dir_entry(entry, dir)?.path();
        if path.is_dir() {
            collect_files(&path, language, out)?;
        } else if language.tracks(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// Every Rust source file under `root` that defines testable behavior — a function with a
/// body, outside any `#[cfg(test)]` module — but carries no inline `#[cfg(test)]` module,
/// sorted. `exempt` holds the rule's `root`-relative paths resolved from config.
pub fn missing_inline_tests(
    root: impl AsRef<Path>,
    exempt: &BTreeSet<String>,
) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files)?;
    files.sort();

    let mut orphans = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading source file `{}`", file.display()))?;
        let ast = syn::parse_file(&source)
            .map_err(|err| anyhow!("parsing `{}`: {err}", file.display()))?;
        let mut visitor = PresenceVisitor::default();
        visitor.visit_file(&ast);
        if !visitor.has_testable_fn || visitor.has_test_module {
            continue;
        }
        let relative = file
            .strip_prefix(root)
            .unwrap_or(file)
            .to_string_lossy()
            .replace('\\', "/");
        if exempt.contains(&relative) {
            continue;
        }
        orphans.push(file.clone());
    }
    // `files` is already sorted, so `orphans` is in order.
    Ok(orphans)
}

/// Recursively collect `*.rs` unit-source files under `dir` into `out`, skipping the
/// non-unit trees — `tests/`, `benches/`, `examples/`, `target/` — and `build.rs`.
pub(crate) fn collect_rust_source_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = crate::walk::dir_entry(entry, dir)?.path();
        if path.is_dir() {
            let skip = matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("tests" | "benches" | "examples" | "target")
            );
            if !skip {
                collect_rust_source_files(&path, out)?;
            }
        } else if has_extension(&path, &["rs"]) && file_name_of(&path) != "build.rs" {
            out.push(path);
        }
    }
    Ok(())
}

/// Answers, for a parsed Rust file, whether it defines testable behavior outside any
/// `#[cfg(test)]` module and whether it carries an inline `#[cfg(test)]` module.
#[derive(Default)]
struct PresenceVisitor {
    test_depth: usize,
    has_testable_fn: bool,
    has_test_module: bool,
}

impl<'ast> Visit<'ast> for PresenceVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let is_test = crate::isolation::has_cfg_test(&node.attrs);
        if is_test {
            self.has_test_module = true;
            self.test_depth += 1;
        }
        visit::visit_item_mod(self, node);
        if is_test {
            self.test_depth -= 1;
        }
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if self.test_depth == 0 && !crate::isolation::has_cfg_test(&node.attrs) {
            self.has_testable_fn = true;
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if self.test_depth == 0 {
            self.has_testable_fn = true;
        }
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if self.test_depth == 0 && node.default.is_some() {
            self.has_testable_fn = true;
        }
        visit::visit_trait_item_fn(self, node);
    }
}

/// `true` when the file's extension is one of `extensions`.
fn has_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| extensions.contains(&ext))
}

/// `true` for a TypeScript declaration file (`*.d.ts` / `*.d.mts` / `*.d.cts`).
fn is_declaration(path: &Path) -> bool {
    let name = file_name_of(path);
    name.ends_with(".d.ts") || name.ends_with(".d.mts") || name.ends_with(".d.cts")
}

/// `true` when any line of Python `source` is neither blank nor a `#` comment.
fn python_has_code(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim_start();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    })
}

/// `true` when Python `base` and `head` tokenize identically. Comments and blank lines
/// never reach a token — the parser's `full-lexer` feature is off — while `Indent` /
/// `Dedent` do, so re-indenting a statement is a code change.
fn python_same_code(base: &str, head: &str) -> bool {
    match (python_tokens(base), python_tokens(head)) {
        (Some(base), Some(head)) => base == head,
        _ => false,
    }
}

/// The token stream of Python `source`, or `None` when `source` is not a valid module.
fn python_tokens(source: &str) -> Option<Vec<Tok>> {
    let tokens: Vec<Tok> = lex(source, Mode::Module)
        .map(|token| token.ok().map(|(tok, _)| tok))
        .collect::<Option<_>>()?;
    // The lexer accepts token sequences the grammar rejects (`def f() return 1` lexes
    // cleanly), so the parse decides validity while the tokens carry the comparison.
    ast::Suite::parse(source, "<source>").ok()?;
    Some(tokens)
}

/// `true` when TypeScript `source` holds anything beyond whitespace and `//` / `/* … */`
/// comments. Any other character — including a string literal's quote — counts as code.
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
    fn python_conftest_is_support_not_a_subject() {
        assert!(Language::Python.is_support(Path::new("conftest.py")));
        assert!(Language::Python.is_support(Path::new("pkg/conftest.py")));
        assert!(!Language::Python.is_support(Path::new("widget.py")));
        assert!(!Language::Python.is_support(Path::new("widget_test.py")));
        assert!(!Language::TypeScript.is_support(Path::new("conftest.ts")));
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
        assert!(Language::TypeScript.has_code("const s = '// not a comment';\n"));
        assert!(Language::TypeScript.has_code("const r = a / b;\n"));
    }

    #[test]
    fn typescript_subject_skips_type_only_modules() {
        let ts = Path::new("aliases.ts");
        assert!(!Language::TypeScript.is_subject("export type Alias = string;\n", ts));
        assert!(!Language::TypeScript.is_subject("export interface Shape { kind: string }\n", ts));
        assert!(!Language::TypeScript.is_subject("import type { A } from './a';\n", ts));
    }

    #[test]
    fn typescript_subject_keeps_anything_with_runtime_behavior() {
        let ts = Path::new("widget.ts");
        assert!(Language::TypeScript.is_subject("export const x = 1;\n", ts));
        assert!(Language::TypeScript
            .is_subject("export type Alias = string;\nexport const x = 1;\n", ts));
        assert!(!Language::TypeScript.is_subject("", ts));
        assert!(!Language::TypeScript.is_subject("// nothing here\n", ts));
    }

    #[test]
    fn python_subject_is_decided_by_code_alone() {
        let py = Path::new("widget.py");
        assert!(Language::Python.is_subject("x = 1\n", py));
        assert!(Language::Python.is_subject("Alias = str\n", py));
        assert!(!Language::Python.is_subject("# just a comment\n", py));
    }

    const PY_WIDGET: &str = "def widget():\n    return 1\n";

    #[test]
    fn python_same_code_ignores_comments_and_formatting() {
        let py = Path::new("widget.py");
        assert!(Language::Python.same_code(
            "# widget helpers\ndef widget():\n    return 1\n",
            "# widget utilities\ndef widget():\n    return 1\n",
            py
        ));
        assert!(Language::Python.same_code(
            "# widget helpers\ndef widget():\n    return 1\n",
            PY_WIDGET,
            py
        ));
        assert!(Language::Python.same_code(PY_WIDGET, "def widget():\n\n    return 1\n", py));
        assert!(Language::Python.same_code("def widget():   \n    return 1   \n", PY_WIDGET, py));
    }

    #[test]
    fn python_same_code_sees_every_edit_the_interpreter_sees() {
        let py = Path::new("widget.py");
        assert!(!Language::Python.same_code(PY_WIDGET, "def widget():\n    return 2\n", py));
        assert!(!Language::Python.same_code(
            "\"\"\"Widget helpers.\"\"\"\ndef widget():\n    return 1\n",
            "\"\"\"Widget utilities.\"\"\"\ndef widget():\n    return 1\n",
            py
        ));
        assert!(!Language::Python.same_code(
            "def widget():\n    return \"one\"\n",
            "def widget():\n    return \"two\"\n",
            py
        ));
        assert!(!Language::Python.same_code(
            "def widget(flag):\n    if flag:\n        count = 1\n    return count\n",
            "def widget(flag):\n    if flag:\n        count = 1\n        return count\n",
            py
        ));
    }

    #[test]
    fn python_same_code_holds_unparseable_content_apart() {
        let py = Path::new("widget.py");
        assert!(!Language::Python.same_code(
            "def widget(:\n    return 1\n",
            "# note\ndef widget(:\n    return 1\n",
            py
        ));
        assert!(!Language::Python.same_code(
            "def widget() return 1\n",
            "# note\ndef widget() return 1\n",
            py
        ));
        assert!(!Language::Python.same_code(PY_WIDGET, "def widget() return 1\n", py));
        assert!(!Language::Python.same_code("def widget() return 1\n", PY_WIDGET, py));
    }

    #[test]
    fn typescript_same_code_reads_the_emitted_module() {
        let ts = Path::new("widget.ts");
        assert!(Language::TypeScript.same_code(
            "// widget factory\nexport const widget = () => 1;\n",
            "export const widget = () => 1;\n",
            ts
        ));
        assert!(!Language::TypeScript.same_code(
            "export const widget = () => 1;\n",
            "export const widget = () => 2;\n",
            ts
        ));
    }

    #[test]
    fn rust_same_code_never_answers_equal() {
        assert!(!Language::Rust.same_code("fn f() {}\n", "fn f() {}\n", Path::new("lib.rs")));
    }

    #[test]
    fn rust_has_no_file_based_colocated_convention() {
        assert!(!Language::Rust.tracks(Path::new("lib.rs")));
        assert!(!Language::Rust.is_test(Path::new("lib_test.rs")));
        assert!(!Language::Rust.has_code("fn main() {}\n"));
        assert_eq!(
            Language::Rust.expected_test_path(Path::new("src/lib.rs")),
            PathBuf::from("src/lib.rs")
        );
    }

    /// `(has_testable_fn, has_test_module)` for a Rust source snippet.
    fn presence(src: &str) -> (bool, bool) {
        let ast = syn::parse_file(src).expect("snippet parses");
        let mut visitor = PresenceVisitor::default();
        visitor.visit_file(&ast);
        (visitor.has_testable_fn, visitor.has_test_module)
    }

    #[test]
    fn rust_presence_free_fn_with_test_module_is_covered() {
        assert_eq!(
            presence(
                "pub fn make(n: u8) -> u8 { n + 1 }\n\
                 #[cfg(test)]\nmod tests { #[test] fn t() {} }\n"
            ),
            (true, true)
        );
    }

    #[test]
    fn rust_presence_free_fn_without_test_module_needs_one() {
        assert_eq!(
            presence("pub fn make(n: u8) -> u8 { n + 1 }\n"),
            (true, false)
        );
    }

    #[test]
    fn rust_presence_type_only_file_is_not_a_subject() {
        assert_eq!(presence("pub struct Point { pub x: u8 }\n"), (false, false));
    }

    #[test]
    fn rust_presence_impl_method_is_testable() {
        assert_eq!(
            presence("pub struct W;\nimpl W { pub fn go(&self) -> u8 { 1 } }\n"),
            (true, false)
        );
    }

    #[test]
    fn rust_presence_trait_default_is_testable_but_bare_signature_is_not() {
        assert_eq!(
            presence("pub trait T { fn d(&self) -> u8 { 1 } }\n"),
            (true, false)
        );
        assert_eq!(
            presence("pub trait T { fn s(&self) -> u8; }\n"),
            (false, false)
        );
    }

    #[test]
    fn rust_presence_test_module_functions_are_not_subjects() {
        assert_eq!(
            presence("#[cfg(test)]\nmod tests { fn helper() {} #[test] fn t() {} }\n"),
            (false, true)
        );
    }

    #[test]
    fn rust_presence_cfg_test_gated_free_fn_is_not_a_subject() {
        assert_eq!(
            presence("#[cfg(test)]\nfn only_in_tests() {}\n"),
            (false, false)
        );
    }
}
