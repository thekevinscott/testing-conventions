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
//! - **`no-environ-mutation`** (#51): direct mutation of `os.environ` —
//!   `os.environ[...] = …`, `del os.environ[...]`, or a mutating method
//!   (`update` / `pop` / `setdefault` / `clear` / `popitem`). Set env via
//!   `patch.dict(os.environ, {...})` instead.
//! - **`no-constant-patch`** (#52): patching a module-global UPPER_CASE constant,
//!   e.g. `patch("pkg.config.CACHE_DIR", …)`. Inject config explicitly. Waivable
//!   per file via the config `exempt` list.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rustpython_ast::Visitor;
use rustpython_parser::ast::{
    self, Arg, Arguments, Constant, Expr, ExprCall, StmtAssign, StmtAsyncFunctionDef,
    StmtAugAssign, StmtDelete, StmtFunctionDef, WithItem,
};
use rustpython_parser::text_size::{TextRange, TextSize};
use rustpython_parser::Parse;

// `Violation` is shared with the Rust `isolation` lint; it lives in `violation`
// and is re-exported here so `testing_conventions::lint::Violation` still resolves.
pub use crate::violation::Violation;

/// Scan the Python test files under `root` and return every lint violation,
/// sorted by `(file, line)` for deterministic output.
///
/// A *Python test file* is `*_test.py`, the legacy `test_*.py`, or
/// `conftest.py` (where fixtures live). Each is parsed and walked. A file that
/// cannot be read or parsed is an error.
pub fn find_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    // The dist's own top-level package, for `no-first-party-patch` (#42). Resolved
    // once for the whole tree; `None` (no declared package) means that rule flags
    // nothing.
    let first_party = first_party_package(root);
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
            first_party: first_party.as_deref(),
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

/// Scan the colocated Python unit tests under `root` and return every
/// `unmocked-collaborator` violation (#42 slice 2): a first-party collaborator a
/// unit test imports without mocking it. The Python arm of `unit isolation`
/// ([`crate::isolation::Language::Python`]).
///
/// A *unit test* here is `*_test.py` / `test_*.py` (not `conftest.py`). First-party
/// is the dist's own package ([`first_party_package`]); a tree with no declared
/// package has no first-party collaborators and so reports nothing.
pub fn find_unit_isolation_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    // Skeleton (#42 slice 2): the detector lands in the green commit; until then
    // the command is wired but reports nothing.
    let _ = root.as_ref();
    Ok(Vec::new())
}

/// Walks one parsed test file, collecting lint violations. Tracks how deep we
/// are inside `@pytest.fixture` functions so `no-inline-patch` can allow patches
/// there while flagging them in test bodies.
struct LintVisitor<'a> {
    file: &'a Path,
    source: &'a str,
    fixture_depth: usize,
    /// The dist's own top-level package (#42), or `None` when undiscoverable.
    first_party: Option<&'a str>,
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
        let is_patch = is_patch_call(&node);
        // `no-inline-patch` (#50): a patch(...) call outside any fixture is a
        // patch in a test body. Inside a fixture it is the right place.
        if is_patch && self.fixture_depth == 0 {
            self.report(
                node.range,
                "no-inline-patch",
                "patch is called inline in a test body; move it into a `pytest.fixture`",
            );
        }
        // `no-constant-patch` (#52): patching a module-global UPPER_CASE constant.
        // Fires regardless of fixture — config constants are usually patched in one.
        if is_patch && patches_constant(&node) {
            self.report(node.range, "no-constant-patch", CONSTANT_PATCH_MSG);
        }
        // `no-first-party-patch` (#42): in an integration test, patching a
        // first-party target — `patch("ourpkg.mod.fn")` — is forbidden; an
        // integration test runs first-party code for real. Fires regardless of
        // fixture (the patch belongs in one); only when the dist's own package is
        // known (`first_party`) and the target's head segment names it.
        if is_patch {
            if let Some(pkg) = self.first_party {
                if patch_string_target(&node).is_some_and(|target| patches_first_party(target, pkg))
                {
                    self.report(node.range, "no-first-party-patch", FIRST_PARTY_PATCH_MSG);
                }
            }
        }
        // `no-environ-mutation` (#51): `os.environ.update(...)` and friends.
        if is_environ_mutation_call(&node) {
            self.report(node.range, "no-environ-mutation", ENVIRON_MUTATION_MSG);
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

    // `no-environ-mutation` (#51): `os.environ[...] = …`, augmented assignment,
    // and `del os.environ[...]`.
    fn visit_stmt_assign(&mut self, node: StmtAssign) {
        if node.targets.iter().any(is_os_environ_subscript) {
            self.report(node.range, "no-environ-mutation", ENVIRON_MUTATION_MSG);
        }
        self.generic_visit_stmt_assign(node);
    }

    fn visit_stmt_aug_assign(&mut self, node: StmtAugAssign) {
        if is_os_environ_subscript(node.target.as_ref()) {
            self.report(node.range, "no-environ-mutation", ENVIRON_MUTATION_MSG);
        }
        self.generic_visit_stmt_aug_assign(node);
    }

    fn visit_stmt_delete(&mut self, node: StmtDelete) {
        if node.targets.iter().any(is_os_environ_subscript) {
            self.report(node.range, "no-environ-mutation", ENVIRON_MUTATION_MSG);
        }
        self.generic_visit_stmt_delete(node);
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

/// Message for the `no-constant-patch` lint.
const CONSTANT_PATCH_MSG: &str = "patches a module-global config constant; inject config explicitly (a consumer that did `from pkg import CONSTANT` snapshots the value at import time and ignores the patch)";

/// Message for the `no-first-party-patch` lint (#42).
const FIRST_PARTY_PATCH_MSG: &str = "patches a first-party target; an integration test must run first-party code for real — only third-party packages and effectful stdlib may be patched";

/// The string-literal first argument of a `patch(...)` call — the dotted target
/// like `"pkg.mod.attr"`. `None` when the first argument isn't a string literal
/// (a non-literal target can't be classified deterministically).
fn patch_string_target(call: &ExprCall) -> Option<&str> {
    if let Some(Expr::Constant(constant)) = call.args.first() {
        if let Constant::Str(target) = &constant.value {
            return Some(target.as_str());
        }
    }
    None
}

/// `true` when a `patch(...)` call's first string argument names a module-global
/// UPPER_CASE constant, e.g. `patch("pkg.config.CACHE_DIR", …)`.
fn patches_constant(call: &ExprCall) -> bool {
    patch_string_target(call)
        .and_then(|target| target.rsplit('.').next())
        .is_some_and(is_upper_constant)
}

/// `true` when a patch `target`'s head dotted segment names the first-party
/// package `pkg`, e.g. `target = "ourpkg.mod.fn"`, `pkg = "ourpkg"` (#42).
fn patches_first_party(target: &str, pkg: &str) -> bool {
    target
        .split('.')
        .next()
        .is_some_and(|head| !head.is_empty() && head == pkg)
}

/// `true` for an ALL-CAPS constant name — letters uppercase, digits and
/// underscores allowed, at least one letter (`CACHE_DIR`, `DEBUG`, `MAX_SIZE`).
fn is_upper_constant(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && name.chars().any(|c| c.is_ascii_uppercase())
}

/// Message for the `no-environ-mutation` lint.
const ENVIRON_MUTATION_MSG: &str =
    "os.environ is mutated directly; set env via `patch.dict(os.environ, {...})` instead";

/// `true` for the expression `os.environ`.
fn is_os_environ(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Attribute(attr)
            if attr.attr.as_str() == "environ"
                && matches!(attr.value.as_ref(), Expr::Name(name) if name.id.as_str() == "os")
    )
}

/// `true` for `os.environ[...]` — a subscript of `os.environ`, the form used as
/// an assignment or `del` target.
fn is_os_environ_subscript(expr: &Expr) -> bool {
    matches!(expr, Expr::Subscript(sub) if is_os_environ(sub.value.as_ref()))
}

/// `true` for a mutating method call on `os.environ` (`os.environ.update(...)`
/// and friends).
fn is_environ_mutation_call(call: &ExprCall) -> bool {
    matches!(
        call.func.as_ref(),
        Expr::Attribute(attr)
            if is_os_environ(attr.value.as_ref()) && is_environ_mutator(attr.attr.as_str())
    )
}

/// `true` for a `dict` method that mutates in place.
fn is_environ_mutator(method: &str) -> bool {
    matches!(
        method,
        "update" | "pop" | "setdefault" | "clear" | "popitem"
    )
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

/// The dist's own top-level import package — the first-party root for
/// `no-first-party-patch` (#42).
///
/// Walk up from `root` to the nearest `pyproject.toml`, read its `[project].name`,
/// and [normalize](normalize_dist_name) it to an import name. Returns `None` when
/// no `pyproject.toml` (with a `[project].name`) is found, so a tree with no
/// declared package flags nothing rather than guess. The walk stops at a `.git`
/// boundary so it can't escape the project into an unrelated `pyproject.toml`.
fn first_party_package(root: &Path) -> Option<String> {
    for dir in root.ancestors() {
        let candidate = dir.join("pyproject.toml");
        if candidate.is_file() {
            return read_project_name(&candidate).map(|name| normalize_dist_name(&name));
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    None
}

/// `[project].name` from a `pyproject.toml`, if present and a string.
fn read_project_name(path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(path).ok()?;
    let value: toml::Value = toml::from_str(&contents).ok()?;
    value
        .get("project")?
        .get("name")?
        .as_str()
        .map(str::to_owned)
}

/// Normalize a distribution name to its import package name: lower-cased, with
/// `-` and `.` mapped to `_` (PEP 503-flavoured — `My-Project` → `my_project`).
fn normalize_dist_name(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['-', '.'], "_")
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
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A throwaway directory, removed on drop — for the `pyproject.toml` discovery.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::temp_dir().join(format!(
                "tc-lint-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir_all(&dir).unwrap();
            TempDir(dir)
        }

        fn write(&self, name: &str, contents: &str) {
            std::fs::write(self.0.join(name), contents).unwrap();
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn normalize_dist_name_maps_to_import_name() {
        assert_eq!(normalize_dist_name("My-Project"), "my_project");
        assert_eq!(normalize_dist_name("ns.pkg"), "ns_pkg");
        assert_eq!(normalize_dist_name("  myproject  "), "myproject");
        assert_eq!(normalize_dist_name("myproject"), "myproject");
    }

    /// Parse `src` (a single expression statement) and return its call.
    fn parse_call(src: &str) -> ExprCall {
        let suite = ast::Suite::parse(src, "t.py").expect("snippet should parse");
        match suite.into_iter().next().expect("one statement") {
            ast::Stmt::Expr(stmt) => match *stmt.value {
                Expr::Call(call) => call,
                other => panic!("expected a call, got {other:?}"),
            },
            other => panic!("expected an expression statement, got {other:?}"),
        }
    }

    #[test]
    fn patch_string_target_only_reads_string_literals() {
        let str_call = parse_call("patch(\"pkg.mod.attr\")\n");
        assert_eq!(patch_string_target(&str_call), Some("pkg.mod.attr"));
        // A non-string literal (`patch(42)`), a name (`patch(target)`), and no args
        // all yield `None` — a non-literal target can't be classified.
        let int_call = parse_call("patch(42)\n");
        assert_eq!(patch_string_target(&int_call), None);
        let name_call = parse_call("patch(target)\n");
        assert_eq!(patch_string_target(&name_call), None);
        let empty_call = parse_call("patch()\n");
        assert_eq!(patch_string_target(&empty_call), None);
    }

    #[test]
    fn patches_first_party_matches_head_segment() {
        assert!(patches_first_party("myproject.ledger.record", "myproject"));
        assert!(patches_first_party("myproject", "myproject"));
        assert!(!patches_first_party("requests.get", "myproject"));
        assert!(!patches_first_party("myproject_extra.x", "myproject"));
        assert!(!patches_first_party("", "myproject"));
        assert!(!patches_first_party(".leading", "myproject"));
    }

    #[test]
    fn first_party_package_reads_pyproject_name() {
        let tree = TempDir::new();
        tree.write(
            "pyproject.toml",
            "[project]\nname = \"My-Project\"\nversion = \"0.0.0\"\n",
        );
        // Normalized to the import name.
        assert_eq!(first_party_package(&tree.0).as_deref(), Some("my_project"));
    }

    #[test]
    fn first_party_package_is_none_without_a_project_name() {
        let tree = TempDir::new();
        // A pyproject with no `[project].name` — found, but no usable package.
        tree.write("pyproject.toml", "[build-system]\nrequires = []\n");
        tree.write(".git", "");
        assert_eq!(first_party_package(&tree.0), None);
    }

    #[test]
    fn first_party_package_is_none_when_absent() {
        // No pyproject.toml anywhere up the (temp) tree → nothing first-party.
        let tree = TempDir::new();
        assert_eq!(first_party_package(&tree.0), None);
    }

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

    #[test]
    fn recognizes_environ_mutators() {
        assert!(is_environ_mutator("update"));
        assert!(is_environ_mutator("pop"));
        assert!(is_environ_mutator("clear"));
        assert!(!is_environ_mutator("get"));
        assert!(!is_environ_mutator("keys"));
    }

    #[test]
    fn recognizes_upper_constants() {
        assert!(is_upper_constant("CACHE_DIR"));
        assert!(is_upper_constant("DEBUG"));
        assert!(is_upper_constant("MAX_2"));
        assert!(!is_upper_constant("cache_dir"));
        assert!(!is_upper_constant("CacheDir"));
        assert!(!is_upper_constant("fetch"));
        assert!(!is_upper_constant(""));
        assert!(!is_upper_constant("_"));
        assert!(!is_upper_constant("123"));
    }
}
