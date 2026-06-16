//! Integration-test lints (issue #19; rules #48–#52) — the `integration lint`
//! command.
//!
//! A *lint* here is a deterministic style/mechanism check on test code, as
//! opposed to the structural `colocated-test` / `coverage` rules. This module hosts
//! the mocking mechanism & style lints; more lints will join them under the
//! same command.
//!
//! Detection is AST-based: each Python test file is parsed with
//! `rustpython_parser` and the tree is walked with a [`Visitor`].
//!
//! Implemented lints:
//! - **`no-monkeypatch`** (#49): a test/fixture function that declares the
//!   `monkeypatch` parameter (pytest's fixture). Patch with `unittest.mock`
//!   wrapped in a `pytest.fixture` instead.
//! - **`no-inline-patch`** (#50): a `patch(...)` / `patch.object(...)` /
//!   `patch.dict(...)` call inside a test body — the `with patch(...)` form or a
//!   bare call. Patches belong in a `pytest.fixture`; a patch *inside* a fixture
//!   is allowed.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rustpython_ast::Visitor;
use rustpython_parser::ast::{
    self, Arg, Arguments, Expr, ExprCall, StmtAsyncFunctionDef, StmtFunctionDef, WithItem,
};
use rustpython_parser::text_size::{TextRange, TextSize};
use rustpython_parser::Parse;

/// A single lint violation found in a test file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Violation {
    /// File the violation was found in.
    pub file: PathBuf,
    /// 1-based line number of the offending construct.
    pub line: usize,
    /// Short lint identifier (e.g. `no-monkeypatch`).
    pub rule: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

/// Scan the Python test files under `root` and return every lint violation,
/// sorted by `(file, line)` for deterministic output.
///
/// A *Python test file* is `*_test.py`, the legacy `test_*.py`, or
/// `conftest.py` (where fixtures live). Each is parsed and walked. A file that
/// cannot be read or parsed is an error.
pub fn find_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_python_test_files(root, &mut files)?;
    files.sort();

    let mut violations = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading test file `{}`", file.display()))?;
        let suite = ast::Suite::parse(&source, &file.to_string_lossy())
            .map_err(|err| anyhow!("parsing `{}`: {err}", file.display()))?;
        let mut visitor = LintVisitor {
            file,
            source: &source,
            fixture_depth: 0,
            violations: Vec::new(),
        };
        for stmt in suite {
            visitor.visit_stmt(stmt);
        }
        violations.append(&mut visitor.violations);
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// Walks one parsed test file, collecting lint violations. Tracks how deep we
/// are inside `@pytest.fixture` functions so `no-inline-patch` can allow patches
/// there while flagging them in test bodies.
struct LintVisitor<'a> {
    file: &'a Path,
    source: &'a str,
    fixture_depth: usize,
    violations: Vec<Violation>,
}

impl LintVisitor<'_> {
    fn report(&mut self, range: TextRange, rule: &'static str, message: &str) {
        self.violations.push(Violation {
            file: self.file.to_path_buf(),
            line: line_of(self.source, range.start()),
            rule,
            message: message.to_string(),
        });
    }

    /// Shared entry for both function kinds: run the parameter lint, then return
    /// whether this function is a fixture (so the caller bumps `fixture_depth`).
    fn enter_function(&mut self, args: &Arguments, decorators: &[Expr], range: TextRange) -> bool {
        // `no-monkeypatch` (#49): the `monkeypatch` parameter is the signal.
        let takes_monkeypatch = args
            .posonlyargs
            .iter()
            .chain(&args.args)
            .chain(&args.kwonlyargs)
            .any(|arg| arg.def.arg.as_str() == "monkeypatch")
            || arg_named(&args.vararg, "monkeypatch")
            || arg_named(&args.kwarg, "monkeypatch");
        if takes_monkeypatch {
            self.report(
                range,
                "no-monkeypatch",
                "test takes pytest's `monkeypatch` fixture; patch with `unittest.mock` wrapped in a `pytest.fixture` instead",
            );
        }

        decorators.iter().any(is_fixture_decorator)
    }
}

impl Visitor for LintVisitor<'_> {
    fn visit_stmt_function_def(&mut self, node: StmtFunctionDef) {
        let is_fixture = self.enter_function(&node.args, &node.decorator_list, node.range);
        if is_fixture {
            self.fixture_depth += 1;
        }
        self.generic_visit_stmt_function_def(node);
        if is_fixture {
            self.fixture_depth -= 1;
        }
    }

    fn visit_stmt_async_function_def(&mut self, node: StmtAsyncFunctionDef) {
        let is_fixture = self.enter_function(&node.args, &node.decorator_list, node.range);
        if is_fixture {
            self.fixture_depth += 1;
        }
        self.generic_visit_stmt_async_function_def(node);
        if is_fixture {
            self.fixture_depth -= 1;
        }
    }

    fn visit_expr_call(&mut self, node: ExprCall) {
        // `no-inline-patch` (#50): a patch(...) call outside any fixture is a
        // patch in a test body. Inside a fixture it is the right place.
        if self.fixture_depth == 0 && is_patch_call(&node) {
            self.report(
                node.range,
                "no-inline-patch",
                "patch is called inline in a test body; move it into a `pytest.fixture`",
            );
        }
        self.generic_visit_expr_call(node);
    }

    // The generated `generic_visit_withitem` is a no-op, so a `with patch(...)`
    // context expression is never walked unless we descend into it here.
    fn visit_withitem(&mut self, node: WithItem) {
        self.visit_expr(node.context_expr);
        if let Some(optional_vars) = node.optional_vars {
            self.visit_expr(*optional_vars);
        }
    }
}

/// `true` when a `*args` / `**kwargs` arg is named `name`.
fn arg_named(arg: &Option<Box<Arg>>, name: &str) -> bool {
    arg.as_ref().is_some_and(|arg| arg.arg.as_str() == name)
}

/// `true` for an `@pytest.fixture` / `@fixture` decorator, with or without a
/// call (`@pytest.fixture(autouse=True)`).
fn is_fixture_decorator(decorator: &Expr) -> bool {
    let target = match decorator {
        Expr::Call(call) => call.func.as_ref(),
        other => other,
    };
    match target {
        Expr::Name(name) => name.id.as_str() == "fixture",
        Expr::Attribute(attr) => attr.attr.as_str() == "fixture",
        _ => false,
    }
}

/// `true` when a call is `patch(...)`, `patch.object(...)`, `patch.dict(...)`, or
/// the same reached through a module (`mock.patch(...)`, `unittest.mock.patch`).
fn is_patch_call(call: &ExprCall) -> bool {
    match call.func.as_ref() {
        Expr::Name(name) => name.id.as_str() == "patch",
        Expr::Attribute(attr) => {
            let name = attr.attr.as_str();
            name == "patch"
                || ((name == "object" || name == "dict") && attr_base_is_patch(attr.value.as_ref()))
        }
        _ => false,
    }
}

/// `true` when an attribute's base resolves to `patch` — the receiver of
/// `patch.object` / `patch.dict`.
fn attr_base_is_patch(expr: &Expr) -> bool {
    match expr {
        Expr::Name(name) => name.id.as_str() == "patch",
        Expr::Attribute(attr) => attr.attr.as_str() == "patch",
        _ => false,
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

/// Recursively collect every Python test file under `dir` into `out`.
fn collect_python_test_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_python_test_files(&path, out)?;
        } else if is_python_test_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` for a file the lints scan: `*_test.py`, legacy `test_*.py`, or
/// `conftest.py`.
fn is_python_test_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name == "conftest.py"
        || name.ends_with("_test.py")
        || (name.starts_with("test_") && name.ends_with(".py"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_python_test_files() {
        assert!(is_python_test_file(Path::new("widget_test.py")));
        assert!(is_python_test_file(Path::new("pkg/widget_test.py")));
        assert!(is_python_test_file(Path::new("test_widget.py")));
        assert!(is_python_test_file(Path::new("conftest.py")));
    }

    #[test]
    fn ignores_non_test_files() {
        assert!(!is_python_test_file(Path::new("widget.py")));
        assert!(!is_python_test_file(Path::new("conftest.pyi")));
        assert!(!is_python_test_file(Path::new("README.md")));
        assert!(!is_python_test_file(Path::new("testing.py")));
    }

    #[test]
    fn line_of_counts_newlines() {
        let src = "a\nb\nc\n";
        assert_eq!(line_of(src, TextSize::from(0)), 1);
        assert_eq!(line_of(src, TextSize::from(2)), 2);
        assert_eq!(line_of(src, TextSize::from(4)), 3);
    }
}
