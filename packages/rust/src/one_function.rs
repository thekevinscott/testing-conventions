//! The `unit one-function-per-file` check: a source file holds at most one module-scope
//! function whose body runs longer than the configured threshold.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use oxc::allocator::Allocator;
use oxc::ast::ast::{Declaration, Expression, Statement, VariableDeclaration};
use oxc::parser::Parser;
use oxc::span::{GetSpan, SourceType};
use rustpython_ast::Ranged;
use rustpython_parser::ast::{self, Constant, Expr, Stmt};
use rustpython_parser::text_size::TextSize;
use rustpython_parser::Parse;
use syn::spanned::Spanned;

pub use crate::violation::Violation;

use crate::colocated_test::Language;

/// The rule id reported for a function sharing its file with another over the threshold.
const RULE: &str = "one-function-per-file";

/// A module-scope function: its name, declaration line, and body code-line count.
#[derive(Debug, PartialEq, Eq)]
struct Function {
    name: String,
    line: usize,
    body_lines: usize,
}

/// A violation for every module-scope function under `root` past the first whose body runs
/// longer than `max_lines`, sorted by `(file, line)`. The first over-threshold function in a
/// file holds it; each later one is a violation naming both.
pub fn find_violations(
    root: impl AsRef<Path>,
    language: Language,
    max_lines: u32,
) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let files = source_files(root, language)?;

    let mut violations = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading source file `{}`", file.display()))?;
        let mut over = functions(&source, file, language)?
            .into_iter()
            .filter(|function| function.body_lines > max_lines as usize);
        let Some(holder) = over.next() else {
            continue;
        };
        for extra in over {
            violations.push(Violation {
                file: file.clone(),
                line: extra.line,
                rule: RULE,
                message: format!(
                    "`{}` runs {} lines, and `{}` already holds this file; \
                     move it to its own module",
                    extra.name, extra.body_lines, holder.name
                ),
            });
        }
    }
    Ok(violations)
}

/// Every file under `root` the rule judges, sorted: the language's source files, minus the
/// test and support files and the suite tiers under `<package root>/tests/`.
fn source_files(root: &Path, language: Language) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    if language == Language::Rust {
        crate::colocated_test::collect_rust_source_files(root, &mut files)?;
        files.sort();
        return Ok(files);
    }
    crate::colocated_test::collect_files(root, language, &mut files)?;
    let manifest = match language {
        Language::Python => "pyproject.toml",
        _ => "package.json",
    };
    if let Some(tests) = crate::tiers::suite_tests_dir(root, manifest) {
        files.retain(|file| !file.starts_with(&tests));
    }
    files.retain(|file| !language.is_test(file) && !language.is_support(file));
    files.sort();
    Ok(files)
}

/// The module-scope functions `source` (the file at `path`) declares, in source order.
fn functions(source: &str, path: &Path, language: Language) -> Result<Vec<Function>> {
    match language {
        Language::Python => python_functions(source, path),
        Language::TypeScript => typescript_functions(source, path),
        Language::Rust => rust_functions(source, path),
    }
}

/// The module-level `def` / `async def` statements of a Python module.
fn python_functions(source: &str, path: &Path) -> Result<Vec<Function>> {
    let suite = ast::Suite::parse(source, &path.to_string_lossy())
        .map_err(|err| anyhow!("parsing `{}`: {err}", path.display()))?;
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for statement in &suite {
        let (name, body, range) = match statement {
            Stmt::FunctionDef(node) => (&node.name, &node.body, node.range),
            Stmt::AsyncFunctionDef(node) => (&node.name, &node.body, node.range),
            _ => continue,
        };
        found.push(Function {
            name: name.to_string(),
            line: line_of(source, range.start()),
            body_lines: python_body_lines(source, &lines, body),
        });
    }
    Ok(found)
}

/// The code lines of a Python function body, the docstring excluded.
fn python_body_lines(source: &str, lines: &[&str], body: &[Stmt]) -> usize {
    let start = body.iter().position(|statement| !is_docstring(statement));
    let Some(start) = start else {
        return 0;
    };
    let first = line_of(source, body[start].range().start());
    let last = line_of(source, body[body.len() - 1].range().end());
    code_lines(lines, first, last, Comment::Hash)
}

/// `true` for a bare string-literal statement — a docstring wherever it leads a body.
fn is_docstring(statement: &Stmt) -> bool {
    let Stmt::Expr(node) = statement else {
        return false;
    };
    matches!(
        node.value.as_ref(),
        Expr::Constant(constant) if matches!(constant.value, Constant::Str(_))
    )
}

/// The module-scope functions of a TypeScript module: `function` declarations and
/// bindings initialized with an arrow or function expression, `export` or not.
fn typescript_functions(source: &str, path: &Path) -> Result<Vec<Function>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path)
        .map_err(|err| anyhow!("reading the source type of `{}`: {err}", path.display()))?;
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return Err(anyhow!("parsing `{}`", path.display()));
    }
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for statement in &parsed.program.body {
        match statement {
            Statement::FunctionDeclaration(node) => {
                push_ts_function(source, &lines, node, &mut found)
            }
            Statement::VariableDeclaration(node) => {
                push_ts_bindings(source, &lines, node, &mut found)
            }
            Statement::ExportNamedDeclaration(node) => match &node.declaration {
                Some(Declaration::FunctionDeclaration(inner)) => {
                    push_ts_function(source, &lines, inner, &mut found)
                }
                Some(Declaration::VariableDeclaration(inner)) => {
                    push_ts_bindings(source, &lines, inner, &mut found)
                }
                _ => {}
            },
            Statement::ExportDefaultDeclaration(node) => {
                if let oxc::ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(inner) =
                    &node.declaration
                {
                    push_ts_function(source, &lines, inner, &mut found)
                }
            }
            _ => {}
        }
    }
    Ok(found)
}

/// Record a TypeScript `function` declaration; a bodyless overload signature is skipped.
fn push_ts_function(
    source: &str,
    lines: &[&str],
    node: &oxc::ast::ast::Function,
    out: &mut Vec<Function>,
) {
    let Some(body) = &node.body else {
        return;
    };
    let name = node
        .id
        .as_ref()
        .map(|id| id.name.to_string())
        .unwrap_or_else(|| "default".to_string());
    out.push(Function {
        name,
        line: line_of(source, TextSize::from(node.span.start)),
        body_lines: ts_body_lines(source, lines, body),
    });
}

/// Record every `const` / `let` / `var` binding initialized with an arrow or function
/// expression.
fn push_ts_bindings(
    source: &str,
    lines: &[&str],
    node: &VariableDeclaration,
    out: &mut Vec<Function>,
) {
    for declarator in &node.declarations {
        let body = match &declarator.init {
            Some(Expression::ArrowFunctionExpression(arrow)) => Some(arrow.body.as_ref()),
            Some(Expression::FunctionExpression(function)) => function.body.as_deref(),
            _ => None,
        };
        let Some(body) = body else {
            continue;
        };
        let Some(name) = declarator.id.get_identifier_name() else {
            continue;
        };
        out.push(Function {
            name: name.to_string(),
            line: line_of(source, TextSize::from(declarator.span.start)),
            body_lines: ts_body_lines(source, lines, body),
        });
    }
}

/// The code lines of a TypeScript function body.
fn ts_body_lines(source: &str, lines: &[&str], body: &oxc::ast::ast::FunctionBody) -> usize {
    let (Some(first), Some(last)) = (body.statements.first(), body.statements.last()) else {
        return 0;
    };
    code_lines(
        lines,
        line_of(source, TextSize::from(first.span().start)),
        line_of(source, TextSize::from(last.span().end)),
        Comment::Slash,
    )
}

/// The `fn` items at the top level of a Rust file.
fn rust_functions(source: &str, path: &Path) -> Result<Vec<Function>> {
    let ast =
        syn::parse_file(source).map_err(|err| anyhow!("parsing `{}`: {err}", path.display()))?;
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for item in &ast.items {
        let syn::Item::Fn(node) = item else {
            continue;
        };
        found.push(Function {
            name: node.sig.ident.to_string(),
            line: node.sig.ident.span().start().line,
            body_lines: rust_body_lines(&lines, &node.block),
        });
    }
    Ok(found)
}

/// The code lines of a Rust function body — the block's statements, never the braces.
fn rust_body_lines(lines: &[&str], block: &syn::Block) -> usize {
    let (Some(first), Some(last)) = (block.stmts.first(), block.stmts.last()) else {
        return 0;
    };
    code_lines(
        lines,
        first.span().start().line,
        last.span().end().line,
        Comment::Slash,
    )
}

/// How a language spells a comment, for the body-line count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Comment {
    /// Python: `#`.
    Hash,
    /// TypeScript and Rust: `//`, and the `/* … */` block form.
    Slash,
}

/// The number of lines in the inclusive 1-based range `first..=last` of `lines` that carry
/// code — blank and comment lines don't count.
fn code_lines(lines: &[&str], first: usize, last: usize, comment: Comment) -> usize {
    lines
        .iter()
        .skip(first.saturating_sub(1))
        .take(last.saturating_sub(first) + 1)
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !is_comment(trimmed, comment)
        })
        .count()
}

/// `true` when a trimmed line carries only a comment. The `* ` form catches a `/* … */`
/// block's interior, which a Rust dereference never matches: no space after the star.
fn is_comment(trimmed: &str, comment: Comment) -> bool {
    match comment {
        Comment::Hash => trimmed.starts_with('#'),
        Comment::Slash => {
            trimmed.starts_with("//")
                || trimmed.starts_with("/*")
                || trimmed == "*"
                || trimmed.starts_with("* ")
                || trimmed.starts_with("*/")
        }
    }
}

/// The 1-based line containing byte `offset` in `source`.
fn line_of(source: &str, offset: TextSize) -> usize {
    let offset = (u32::from(offset) as usize).min(source.len());
    source.as_bytes()[..offset]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count()
        + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `(name, body_lines)` pairs of a Python module's module-scope functions.
    fn python(source: &str) -> Vec<(String, usize)> {
        python_functions(source, Path::new("widget.py"))
            .expect("the snippet parses")
            .into_iter()
            .map(|function| (function.name, function.body_lines))
            .collect()
    }

    /// The `(name, body_lines)` pairs of a TypeScript module's module-scope functions.
    fn typescript(source: &str) -> Vec<(String, usize)> {
        typescript_functions(source, Path::new("widget.ts"))
            .expect("the snippet parses")
            .into_iter()
            .map(|function| (function.name, function.body_lines))
            .collect()
    }

    /// The `(name, body_lines)` pairs of a Rust file's top-level `fn` items.
    fn rust(source: &str) -> Vec<(String, usize)> {
        rust_functions(source, Path::new("widget.rs"))
            .expect("the snippet parses")
            .into_iter()
            .map(|function| (function.name, function.body_lines))
            .collect()
    }

    #[test]
    fn python_counts_module_level_defs_and_their_body_lines() {
        let found = python(
            "def alpha(value):\n    total = value + 1\n    return total\n\n\
             async def beta(value):\n    return value\n",
        );
        assert_eq!(
            found,
            vec![("alpha".to_string(), 2), ("beta".to_string(), 1)]
        );
    }

    #[test]
    fn python_skips_methods_and_nested_functions() {
        let found = python(
            "class Widget:\n    def grow(self):\n        self.size += 1\n        return self.size\n\n\
             def build(values):\n    def inner(value):\n        return value * 2\n\n    return inner\n",
        );
        assert_eq!(found, vec![("build".to_string(), 3)]);
    }

    #[test]
    fn python_excludes_the_docstring_blank_lines_and_comments() {
        let found = python(
            "def described(value):\n    \"\"\"Return the value unchanged.\"\"\"\n\
             \x20   # the identity is the whole contract\n\n    return value\n",
        );
        assert_eq!(found, vec![("described".to_string(), 1)]);
    }

    #[test]
    fn python_reports_a_decorated_function_by_name() {
        let found = python("@cache\ndef alpha(value):\n    return value\n");
        assert_eq!(found, vec![("alpha".to_string(), 1)]);
    }

    #[test]
    fn python_counts_an_empty_body_as_no_lines() {
        let found = python("def stub():\n    \"\"\"Nothing yet.\"\"\"\n");
        assert_eq!(found, vec![("stub".to_string(), 0)]);
    }

    #[test]
    fn typescript_counts_declarations_and_function_bound_bindings() {
        let found = typescript(
            "export function alpha(value: number): number {\n  const total = value + 1;\n  return total;\n}\n\
             const beta = (value: number): number => value * 2;\n\
             export const gamma = function (value: number): number {\n  return value;\n};\n",
        );
        assert_eq!(
            found,
            vec![
                ("alpha".to_string(), 2),
                ("beta".to_string(), 1),
                ("gamma".to_string(), 1),
            ]
        );
    }

    #[test]
    fn typescript_skips_methods_nested_arrows_and_non_function_bindings() {
        let found = typescript(
            "const SIZE = 3;\n\
             export class Widget {\n  grow(amount: number): number {\n    return amount;\n  }\n}\n\
             export function build(values: number[]): number[] {\n  const inner = (v: number) => v * 2;\n  return values.map(inner);\n}\n",
        );
        assert_eq!(found, vec![("build".to_string(), 2)]);
    }

    #[test]
    fn typescript_counts_an_export_default_function() {
        let found = typescript(
            "export default function alpha(value: number): number {\n  const total = value + 1;\n  return total;\n}\n",
        );
        assert_eq!(found, vec![("alpha".to_string(), 2)]);
    }

    #[test]
    fn typescript_skips_an_overload_signature() {
        let found = typescript(
            "export function alpha(value: number): number;\n\
             export function alpha(value: string): string;\n\
             export function alpha(value: unknown): unknown {\n  const echoed = value;\n  return echoed;\n}\n",
        );
        assert_eq!(found, vec![("alpha".to_string(), 2)]);
    }

    #[test]
    fn typescript_excludes_comment_lines_from_the_body() {
        let found = typescript(
            "export function described(value: number): number {\n  // the identity is the whole contract\n\n  return value;\n}\n",
        );
        assert_eq!(found, vec![("described".to_string(), 1)]);
    }

    #[test]
    fn rust_counts_top_level_items_only() {
        let found = rust(
            "pub struct Widget;\n\
             impl Widget {\n    pub fn grow(&self) -> u8 {\n        1\n    }\n}\n\
             pub fn build(values: &[u8]) -> u8 {\n    fn inner(v: u8) -> u8 {\n        v * 2\n    }\n    inner(values[0])\n}\n",
        );
        assert_eq!(found, vec![("build".to_string(), 4)]);
    }

    #[test]
    fn rust_skips_functions_in_an_inline_test_module() {
        let found = rust(
            "pub fn ratio(a: u8, b: u8) -> u8 {\n    (a + b) / 2\n}\n\
             #[cfg(test)]\nmod tests {\n    #[test]\n    fn halves() {\n        let x = 1;\n        assert_eq!(x, 1);\n    }\n}\n",
        );
        assert_eq!(found, vec![("ratio".to_string(), 1)]);
    }

    #[test]
    fn rust_excludes_doc_comments_and_body_comments() {
        let found = rust(
            "/// Return the value unchanged.\npub fn described(value: u8) -> u8 {\n    // the identity is the whole contract\n\n    value\n}\n",
        );
        assert_eq!(found, vec![("described".to_string(), 1)]);
    }

    #[test]
    fn rust_counts_an_empty_body_as_no_lines() {
        let found = rust("pub fn stub() {}\n");
        assert_eq!(found, vec![("stub".to_string(), 0)]);
    }

    #[test]
    fn typescript_counts_a_plain_function_declaration() {
        let found = typescript(
            "function alpha(value: number): number {\n  const total = value + 1;\n  return total;\n}\n",
        );
        assert_eq!(found, vec![("alpha".to_string(), 2)]);
    }

    #[test]
    fn typescript_skips_non_function_defaults_and_imports() {
        let found = typescript(
            "import { x } from './x';\nexport default class Widget {}\n\
             const beta = (value: number): number => value;\n",
        );
        assert_eq!(found, vec![("beta".to_string(), 1)]);
    }

    #[test]
    fn typescript_skips_a_destructured_function_binding() {
        let found = typescript("const { a } = () => {};\n");
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn typescript_counts_an_empty_arrow_body_as_no_lines() {
        let found = typescript("const stub = () => {};\n");
        assert_eq!(found, vec![("stub".to_string(), 0)]);
    }

    #[test]
    fn typescript_parse_error_is_reported() {
        let err = typescript_functions("const x = ;\n", Path::new("bad.ts")).unwrap_err();
        assert!(err.to_string().contains("parsing"), "got: {err}");
    }

    fn unique_tmp(slug: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "tc-one-function-{slug}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn suite_tier_files_are_not_judged() {
        let root = unique_tmp("suite");
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(root.join("pyproject.toml"), "[project]\nname = \"w\"\n").unwrap();
        let two_functions = "def alpha():\n    return 1\n\ndef beta():\n    return 2\n";
        std::fs::write(root.join("widget.py"), two_functions).unwrap();
        std::fs::write(root.join("tests").join("helper.py"), two_functions).unwrap();
        let found = find_violations(&root, Language::Python, 0).expect("the tree scans");
        assert_eq!(found.len(), 1, "got: {found:?}");
        assert!(found[0].file.ends_with("widget.py"), "got: {found:?}");
    }

    #[test]
    fn a_root_without_a_manifest_judges_every_source_file() {
        let root = unique_tmp("no-manifest");
        std::fs::create_dir_all(&root).unwrap();
        let two_functions = "def alpha():\n    return 1\n\ndef beta():\n    return 2\n";
        std::fs::write(root.join("widget.py"), two_functions).unwrap();
        let found = find_violations(&root, Language::Python, 0).expect("the tree scans");
        assert_eq!(found.len(), 1, "got: {found:?}");
    }

    #[test]
    fn a_typescript_suite_tier_is_not_judged() {
        let root = unique_tmp("ts-suite");
        std::fs::create_dir_all(root.join("tests")).unwrap();
        std::fs::write(root.join("package.json"), "{ \"name\": \"w\" }\n").unwrap();
        let two = "const alpha = () => {\n  return 1;\n};\nconst beta = () => {\n  return 2;\n};\n";
        std::fs::write(root.join("widget.ts"), two).unwrap();
        std::fs::write(root.join("tests").join("helper.ts"), two).unwrap();
        let found = find_violations(&root, Language::TypeScript, 0).expect("the tree scans");
        assert_eq!(found.len(), 1, "got: {found:?}");
        assert!(found[0].file.ends_with("widget.ts"), "got: {found:?}");
    }

    #[test]
    fn a_rust_root_collects_only_rust_sources() {
        let root = unique_tmp("rust-root");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("widget.rs"), "pub fn one() -> u8 {\n    1\n}\n").unwrap();
        let found = find_violations(&root, Language::Rust, 0).expect("the tree scans");
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn an_unreadable_source_names_the_file() {
        let root = unique_tmp("unreadable");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("widget.rs"), [0xFF, 0xFE]).unwrap();
        let err = find_violations(&root, Language::Rust, 0).unwrap_err();
        assert!(
            err.to_string().contains("reading source file"),
            "got: {err}"
        );
    }

    #[test]
    fn an_unparsable_python_source_names_the_file() {
        let err = python_functions("def broken(:\n", Path::new("widget.py")).unwrap_err();
        assert!(err.to_string().contains("parsing"), "got: {err}");
    }

    #[test]
    fn an_extension_without_a_source_type_is_an_error() {
        let err = typescript_functions("", Path::new("widget.txt")).unwrap_err();
        assert!(
            err.to_string().contains("reading the source type"),
            "got: {err}"
        );
    }

    #[test]
    fn an_unnamed_default_export_is_reported_as_default() {
        let found =
            typescript("export default function (value: number): number {\n  return value;\n}\n");
        assert_eq!(found, vec![("default".to_string(), 1)]);
    }

    #[test]
    fn an_unparsable_rust_source_names_the_file() {
        let err = rust_functions("fn broken( {\n", Path::new("widget.rs")).unwrap_err();
        assert!(err.to_string().contains("parsing"), "got: {err}");
    }

    #[test]
    fn code_lines_skips_blank_and_comment_lines() {
        let lines = vec![
            "let a = 1;",
            "",
            "// note",
            "/* block",
            " * inner",
            " */",
            "a",
        ];
        assert_eq!(code_lines(&lines, 1, 7, Comment::Slash), 2);
    }

    #[test]
    fn is_comment_keeps_a_rust_dereference_as_code() {
        assert!(!is_comment("*counter += 1;", Comment::Slash));
        assert!(is_comment("* inner", Comment::Slash));
        assert!(is_comment("# note", Comment::Hash));
        assert!(!is_comment("value = 1", Comment::Hash));
    }

    #[test]
    fn line_of_counts_newlines_before_the_offset() {
        let source = "a\nb\nc";
        assert_eq!(line_of(source, TextSize::from(0)), 1);
        assert_eq!(line_of(source, TextSize::from(2)), 2);
        assert_eq!(line_of(source, TextSize::from(4)), 3);
    }
}
