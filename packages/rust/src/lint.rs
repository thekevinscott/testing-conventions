//! Integration-test lints (issue #19; rules #48–#52) — the `integration lint`
//! command.
//!
//! A *lint* here is a deterministic style/mechanism check on test code, as
//! opposed to the structural `location` / `coverage` rules. This module hosts
//! the mocking mechanism & style lints; more lints will join them under the
//! same command.
//!
//! Detection is AST-based: each Python test file is parsed with
//! `rustpython_parser` and the tree is walked.
//!
//! Implemented lints:
//! - **`no-monkeypatch`** (#49): a test/fixture function that declares the
//!   `monkeypatch` parameter (pytest's fixture). Patch with `unittest.mock`
//!   wrapped in a `pytest.fixture` instead.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rustpython_parser::ast::{self, Arguments, Stmt};
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
/// `conftest.py` (where fixtures live). Each is parsed and its function
/// definitions — at any nesting depth (`pytest-describe` nests them, classes
/// hold them) — are checked against the lints. A file that cannot be read or
/// parsed is an error.
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
        check_suite(&suite, file, &source, &mut violations);
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// Walk a block of statements, descending into the bodies that can hold a test
/// function — nested `def`s (`pytest-describe`) and classes — and check every
/// function definition's parameters.
fn check_suite(stmts: &[Stmt], file: &Path, source: &str, out: &mut Vec<Violation>) {
    for stmt in stmts {
        match stmt {
            Stmt::FunctionDef(f) => {
                check_arguments(&f.args, f.range, file, source, out);
                check_suite(&f.body, file, source, out);
            }
            Stmt::AsyncFunctionDef(f) => {
                check_arguments(&f.args, f.range, file, source, out);
                check_suite(&f.body, file, source, out);
            }
            Stmt::ClassDef(c) => check_suite(&c.body, file, source, out),
            _ => {}
        }
    }
}

/// `no-monkeypatch` (#49): flag a function that declares the `monkeypatch`
/// parameter — the definitive signal that a test/fixture uses pytest's
/// `monkeypatch` fixture.
fn check_arguments(
    args: &Arguments,
    range: TextRange,
    file: &Path,
    source: &str,
    out: &mut Vec<Violation>,
) {
    let takes_monkeypatch = args
        .posonlyargs
        .iter()
        .chain(args.args.iter())
        .chain(args.kwonlyargs.iter())
        .any(|arg| arg.def.arg.as_str() == "monkeypatch")
        || vararg_is_monkeypatch(&args.vararg)
        || vararg_is_monkeypatch(&args.kwarg);

    if takes_monkeypatch {
        out.push(Violation {
            file: file.to_path_buf(),
            line: line_of(source, range.start()),
            rule: "no-monkeypatch",
            message:
                "test takes pytest's `monkeypatch` fixture; patch with `unittest.mock` wrapped in a `pytest.fixture` instead"
                    .to_string(),
        });
    }
}

/// `true` when a `*args` / `**kwargs` arg is named `monkeypatch`.
fn vararg_is_monkeypatch(arg: &Option<Box<ast::Arg>>) -> bool {
    arg.as_ref()
        .is_some_and(|arg| arg.arg.as_str() == "monkeypatch")
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
