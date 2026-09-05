//! The Python mocking mechanism and style lints behind `integration lint`, plus the Python
//! arm of `unit lint`. Each test file is parsed with `rustpython_parser` and walked with a
//! [`Visitor`]; the rules themselves are documented under `docs/reference/checks/`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use rustpython_ast::Visitor;
use rustpython_parser::ast::{
    self, Arg, Arguments, Constant, Expr, ExprCall, StmtAssign, StmtAsyncFunctionDef,
    StmtAugAssign, StmtDelete, StmtFunctionDef, StmtIf, StmtImport, StmtImportFrom, WithItem,
};
use rustpython_parser::text_size::{TextRange, TextSize};
use rustpython_parser::Parse;

// Re-exported so `testing_conventions::lint::Violation` still resolves.
pub use crate::violation::Violation;

/// Every lint violation in the Python test files under `root`, sorted by `(file, line)`. A
/// *Python test file* is `*_test.py` or `conftest.py`, where fixtures live; a legacy
/// `test_*.py` is ordinary source. A file that cannot be read or parsed is an error.
pub fn find_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    // Resolved once for the whole tree; `None` means `no-first-party-patch` flags nothing.
    let first_party = first_party_package(root);
    let mut files = Vec::new();
    collect_python_files(root, &mut files, is_python_test_file)?;
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
            imports: HashMap::new(),
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

const UNKNOWN_TIER_MSG: &str = "test file sits under `tests/` outside the standard suite tiers; \
     a suite lives in `tests/integration/` or `tests/e2e/`";

/// Every lint violation in `package_root`'s suite tiers, sorted by `(file, line)`.
/// `tests/integration/` and `tests/e2e/` both run first-party code for real; a `*_test.py`
/// under `tests/` outside them is `unknown-tier` rather than silently unscanned.
pub fn find_suite_violations(package_root: &Path) -> Result<Vec<Violation>> {
    let tests = package_root.join("tests");
    let mut violations = Vec::new();
    let tiers = ["integration", "e2e"].map(|tier| tests.join(tier));
    for tier in &tiers {
        if tier.is_dir() {
            violations.extend(find_violations(tier)?);
        }
    }
    if tests.is_dir() {
        let mut strays = Vec::new();
        collect_python_files(&tests, &mut strays, is_python_unit_test_file)?;
        strays.retain(|file| !tiers.iter().any(|tier| file.starts_with(tier)));
        for file in strays {
            violations.push(Violation {
                file,
                line: 1,
                rule: "unknown-tier",
                message: UNKNOWN_TIER_MSG.to_string(),
            });
        }
    }
    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// Every `unmocked-collaborator` violation under `root` — a collaborator a `*_test.py`
/// imports without mocking it — sorted by `(file, line)`. First-party is the dist's own
/// package ([`first_party_package`]); a tree that declares none reports nothing.
pub fn find_unit_isolation_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    // Resolved from the same `pyproject.toml` as `first_party_package`, so a tree with no
    // manifest exits here and the package name is the only remaining unknown.
    let Some(tests) = crate::tiers::suite_tests_dir(root, "pyproject.toml") else {
        return Ok(Vec::new());
    };
    let Some(first_party) = first_party_package(root) else {
        return Ok(Vec::new());
    };
    let mut files = Vec::new();
    collect_python_files(root, &mut files, is_python_unit_test_file)?;
    // The suite tiers run first-party code for real, so their files are never unit subjects.
    files.retain(|file| !file.starts_with(&tests));
    files.sort();

    let mut violations = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading test file `{}`", file.display()))?;
        let suite = ast::Suite::parse(&source, &file.to_string_lossy())
            .map_err(|err| anyhow!("parsing `{}`: {err}", file.display()))?;
        let base = unit_under_test_base(file);
        let mut visitor = UnitIsolationVisitor {
            source: &source,
            first_party: &first_party,
            base: &base,
            type_checking_depth: 0,
            imports: Vec::new(),
            patch_targets: Vec::new(),
        };
        for stmt in suite {
            visitor.visit_stmt(stmt);
        }
        for import in &visitor.imports {
            if import.is_uut || import.is_mocked(&visitor.patch_targets) {
                continue;
            }
            violations.push(Violation {
                file: file.to_path_buf(),
                line: import.line,
                rule: "unmocked-collaborator",
                message: format!(
                    "unit test imports `{}` without mocking it — a unit test isolates the \
                     unit under test, so mock every collaborator (patch it by string in a \
                     fixture)",
                    import.display
                ),
            });
        }
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// One import seen in a unit test, with what it takes to decide whether it is mocked.
struct ImportRecord {
    /// The module path to name in the message (`myproject.ledger`, `.ledger`).
    display: String,
    line: usize,
    is_uut: bool,
    /// For `from X import a, b` — the bound symbols, each of which must be mocked.
    symbols: Vec<String>,
    /// For an **absolute** `from X import a, b` — the source module `X`, which a mocking
    /// patch must name. `None` for a relative `from`-import, which has no module to compare.
    source: Option<String>,
    /// For `import X.Y` — the module path (a patch reaching into it counts as a mock).
    module: Option<String>,
}

impl ImportRecord {
    /// `true` when some `patch("…")` target mocks this import: a plain `import X.Y` by any
    /// patch reaching into `X.Y`, a `from X import a, b` only when **every** bound symbol
    /// is patched at `X` itself.
    fn is_mocked(&self, patch_targets: &[String]) -> bool {
        if let Some(module) = &self.module {
            let prefix = format!("{module}.");
            return patch_targets
                .iter()
                .any(|target| target == module || target.starts_with(&prefix));
        }
        if self.symbols.is_empty() {
            return false;
        }
        self.symbols.iter().all(|symbol| {
            patch_targets
                .iter()
                .any(|target| self.symbol_is_mocked(target, symbol))
        })
    }

    /// `true` when `target`'s last dotted segment is `symbol` and — for an absolute import
    /// — its module path is the import's own [`source`](Self::source).
    fn symbol_is_mocked(&self, target: &str, symbol: &str) -> bool {
        let Some(module) = target.strip_suffix(&format!(".{symbol}")) else {
            return false;
        };
        match &self.source {
            Some(source) => module == source,
            None => true,
        }
    }
}

/// Walks one unit test, collecting its imports and every `patch("…")` string target so
/// [`find_unit_isolation_violations`] can pair them. An `if TYPE_CHECKING:` import is erased
/// at runtime and skipped.
struct UnitIsolationVisitor<'a> {
    source: &'a str,
    first_party: &'a str,
    base: &'a str,
    type_checking_depth: usize,
    imports: Vec<ImportRecord>,
    patch_targets: Vec<String>,
}

impl Visitor for UnitIsolationVisitor<'_> {
    fn visit_stmt_import(&mut self, node: StmtImport) {
        if self.type_checking_depth == 0 {
            let line = line_of(self.source, node.range.start());
            for alias in &node.names {
                let module = alias.name.as_str();
                if is_checked_import(import_head(module), self.first_party) {
                    self.imports.push(ImportRecord {
                        display: module.to_string(),
                        line,
                        is_uut: last_segment(module) == self.base,
                        symbols: Vec::new(),
                        source: None,
                        module: Some(module.to_string()),
                    });
                }
            }
        }
        self.generic_visit_stmt_import(node);
    }

    fn visit_stmt_import_from(&mut self, node: StmtImportFrom) {
        if self.type_checking_depth == 0 {
            let level = relative_level(&node);
            let module = node.module.as_ref().map(|m| m.as_str());
            // A relative import is first-party; an absolute one is judged by its head.
            let should_check = level > 0
                || module.is_some_and(|m| is_checked_import(import_head(m), self.first_party));
            if should_check {
                let line = line_of(self.source, node.range.start());
                let dots = ".".repeat(level);
                match module {
                    // `from <module> import a, b` — the bound symbols are the collaborators.
                    Some(module) => self.imports.push(ImportRecord {
                        display: format!("{dots}{module}"),
                        line,
                        is_uut: last_segment(module) == self.base,
                        symbols: node.names.iter().map(|a| a.name.to_string()).collect(),
                        source: (level == 0).then(|| module.to_string()),
                        module: None,
                    }),
                    // `from . import sub` — each name is a submodule.
                    None => {
                        // In `__init___test.py` a bare `from . import …` names the
                        // package's own re-export surface — the unit under test itself.
                        let barrel_sut = self.base == "__init__" && level == 1;
                        for alias in &node.names {
                            let name = alias.name.as_str();
                            self.imports.push(ImportRecord {
                                display: format!("{dots}{name}"),
                                line,
                                is_uut: barrel_sut || name == self.base,
                                symbols: vec![name.to_string()],
                                source: None,
                                module: None,
                            });
                        }
                    }
                }
            }
        }
        self.generic_visit_stmt_import_from(node);
    }

    fn visit_expr_call(&mut self, node: ExprCall) {
        if is_patch_call(&node) {
            if let Some(target) = patch_string_target(&node) {
                self.patch_targets.push(target.to_string());
            }
        }
        self.generic_visit_expr_call(node);
    }

    fn visit_stmt_if(&mut self, node: StmtIf) {
        // An `if TYPE_CHECKING:` body is type-only; its runtime `else` is still walked.
        if is_type_checking(node.test.as_ref()) {
            self.type_checking_depth += 1;
            for stmt in node.body {
                self.visit_stmt(stmt);
            }
            self.type_checking_depth -= 1;
            for stmt in node.orelse {
                self.visit_stmt(stmt);
            }
        } else {
            self.generic_visit_stmt_if(node);
        }
    }
}

/// The leading dotted segment of a module path (`myproject.db` → `myproject`).
fn import_head(module: &str) -> &str {
    module.split('.').next().unwrap_or(module)
}

/// `true` when an import head names a checked collaborator — the dist package, a third-party
/// package, or effectful stdlib. The test framework and pure stdlib are not collaborators.
fn is_checked_import(head: &str, first_party: &str) -> bool {
    if head == first_party {
        return true;
    }
    if TEST_FRAMEWORK.contains(&head) {
        return false;
    }
    if EFFECTFUL_STDLIB.contains(&head) {
        return true;
    }
    if STDLIB_MODULES.contains(&head) {
        return false;
    }
    true // an unrecognized head is a third-party package
}

/// The test harness, never a collaborator. `unittest` is stdlib; these are the rest.
const TEST_FRAMEWORK: &[&str] = &["pytest", "_pytest", "mock"];

/// Standard-library modules that are **effectful at the head**. Dual-nature heads (`os`,
/// `pathlib`, `datetime`, `time`, `io`, `logging`, `threading`) are excluded: a pure use
/// can't be told from an effectful one at the import, so the patch convention catches those.
const EFFECTFUL_STDLIB: &[&str] = &[
    "asynchat",
    "asyncore",
    "ctypes",
    "curses",
    "dbm",
    "fcntl",
    "ftplib",
    "imaplib",
    "mmap",
    "msvcrt",
    "multiprocessing",
    "nis",
    "nntplib",
    "ossaudiodev",
    "poplib",
    "pty",
    "random",
    "secrets",
    "select",
    "selectors",
    "signal",
    "smtpd",
    "smtplib",
    "socket",
    "socketserver",
    "spwd",
    "sqlite3",
    "ssl",
    "subprocess",
    "syslog",
    "telnetlib",
    "termios",
    "tty",
    "webbrowser",
    "winreg",
    "winsound",
];

/// Python's `sys.stdlib_module_names`, which tells pure stdlib from a third-party package.
/// The [`EFFECTFUL_STDLIB`] subset is what is actually flagged.
const STDLIB_MODULES: &[&str] = &[
    "__future__",
    "_abc",
    "_aix_support",
    "_ast",
    "_asyncio",
    "_bisect",
    "_blake2",
    "_bz2",
    "_codecs",
    "_codecs_cn",
    "_codecs_hk",
    "_codecs_iso2022",
    "_codecs_jp",
    "_codecs_kr",
    "_codecs_tw",
    "_collections",
    "_collections_abc",
    "_compat_pickle",
    "_compression",
    "_contextvars",
    "_crypt",
    "_csv",
    "_ctypes",
    "_curses",
    "_curses_panel",
    "_datetime",
    "_dbm",
    "_decimal",
    "_elementtree",
    "_frozen_importlib",
    "_frozen_importlib_external",
    "_functools",
    "_gdbm",
    "_hashlib",
    "_heapq",
    "_imp",
    "_io",
    "_json",
    "_locale",
    "_lsprof",
    "_lzma",
    "_markupbase",
    "_md5",
    "_msi",
    "_multibytecodec",
    "_multiprocessing",
    "_opcode",
    "_operator",
    "_osx_support",
    "_overlapped",
    "_pickle",
    "_posixshmem",
    "_posixsubprocess",
    "_py_abc",
    "_pydatetime",
    "_pydecimal",
    "_pyio",
    "_pylong",
    "_queue",
    "_random",
    "_scproxy",
    "_sha1",
    "_sha2",
    "_sha3",
    "_signal",
    "_sitebuiltins",
    "_socket",
    "_sqlite3",
    "_sre",
    "_ssl",
    "_stat",
    "_statistics",
    "_string",
    "_strptime",
    "_struct",
    "_symtable",
    "_thread",
    "_threading_local",
    "_tkinter",
    "_tokenize",
    "_tracemalloc",
    "_typing",
    "_uuid",
    "_warnings",
    "_weakref",
    "_weakrefset",
    "_winapi",
    "_zoneinfo",
    "abc",
    "aifc",
    "antigravity",
    "argparse",
    "array",
    "ast",
    "asynchat",
    "asyncio",
    "asyncore",
    "atexit",
    "audioop",
    "base64",
    "bdb",
    "binascii",
    "bisect",
    "builtins",
    "bz2",
    "cProfile",
    "calendar",
    "cgi",
    "cgitb",
    "chunk",
    "cmath",
    "cmd",
    "code",
    "codecs",
    "codeop",
    "collections",
    "colorsys",
    "compileall",
    "concurrent",
    "configparser",
    "contextlib",
    "contextvars",
    "copy",
    "copyreg",
    "crypt",
    "csv",
    "ctypes",
    "curses",
    "dataclasses",
    "datetime",
    "dbm",
    "decimal",
    "difflib",
    "dis",
    "distutils",
    "doctest",
    "email",
    "encodings",
    "ensurepip",
    "enum",
    "errno",
    "faulthandler",
    "fcntl",
    "filecmp",
    "fileinput",
    "fnmatch",
    "fractions",
    "ftplib",
    "functools",
    "gc",
    "genericpath",
    "getopt",
    "getpass",
    "gettext",
    "glob",
    "graphlib",
    "grp",
    "gzip",
    "hashlib",
    "heapq",
    "hmac",
    "html",
    "http",
    "idlelib",
    "imaplib",
    "imghdr",
    "imp",
    "importlib",
    "inspect",
    "io",
    "ipaddress",
    "itertools",
    "json",
    "keyword",
    "lib2to3",
    "linecache",
    "locale",
    "logging",
    "lzma",
    "mailbox",
    "mailcap",
    "marshal",
    "math",
    "mimetypes",
    "mmap",
    "modulefinder",
    "msilib",
    "msvcrt",
    "multiprocessing",
    "netrc",
    "nis",
    "nntplib",
    "nt",
    "ntpath",
    "nturl2path",
    "numbers",
    "opcode",
    "operator",
    "optparse",
    "os",
    "ossaudiodev",
    "pathlib",
    "pdb",
    "pickle",
    "pickletools",
    "pipes",
    "pkgutil",
    "platform",
    "plistlib",
    "poplib",
    "posix",
    "posixpath",
    "pprint",
    "profile",
    "pstats",
    "pty",
    "pwd",
    "py_compile",
    "pyclbr",
    "pydoc",
    "pydoc_data",
    "pyexpat",
    "queue",
    "quopri",
    "random",
    "re",
    "readline",
    "reprlib",
    "resource",
    "rlcompleter",
    "runpy",
    "sched",
    "secrets",
    "select",
    "selectors",
    "shelve",
    "shlex",
    "shutil",
    "signal",
    "site",
    "smtpd",
    "smtplib",
    "sndhdr",
    "socket",
    "socketserver",
    "spwd",
    "sqlite3",
    "sre_compile",
    "sre_constants",
    "sre_parse",
    "ssl",
    "stat",
    "statistics",
    "string",
    "stringprep",
    "struct",
    "subprocess",
    "sunau",
    "symtable",
    "sys",
    "sysconfig",
    "syslog",
    "tabnanny",
    "tarfile",
    "telnetlib",
    "tempfile",
    "termios",
    "textwrap",
    "this",
    "threading",
    "time",
    "timeit",
    "tkinter",
    "token",
    "tokenize",
    "tomllib",
    "trace",
    "traceback",
    "tracemalloc",
    "tty",
    "turtle",
    "turtledemo",
    "types",
    "typing",
    "unicodedata",
    "unittest",
    "urllib",
    "uu",
    "uuid",
    "venv",
    "warnings",
    "wave",
    "weakref",
    "webbrowser",
    "winreg",
    "winsound",
    "wsgiref",
    "xdrlib",
    "xml",
    "xmlrpc",
    "zipapp",
    "zipfile",
    "zipimport",
    "zlib",
    "zoneinfo",
];

/// The trailing dotted segment of a module path (`myproject.db` → `db`).
fn last_segment(module: &str) -> &str {
    module.rsplit('.').next().unwrap_or(module)
}

/// The number of leading dots on a `from`-import: `from ..pkg import x` → 2, absolute → 0.
fn relative_level(node: &StmtImportFrom) -> usize {
    node.level.map_or(0, |level| level.to_usize())
}

/// `true` for `TYPE_CHECKING` / `typing.TYPE_CHECKING`, the guard over type-only imports.
fn is_type_checking(test: &Expr) -> bool {
    match test {
        Expr::Name(name) => name.id.as_str() == "TYPE_CHECKING",
        Expr::Attribute(attr) => attr.attr.as_str() == "TYPE_CHECKING",
        _ => false,
    }
}

/// The unit-under-test base name for a test file: `widget_test.py` → `widget`. Only
/// `*_test.py` reaches here, so stripping the `_test` suffix is all it takes.
fn unit_under_test_base(file: &Path) -> String {
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let stem = name.strip_suffix(".py").unwrap_or(name);
    stem.strip_suffix("_test").unwrap_or(stem).to_string()
}

/// Walks one test file, collecting lint violations. `fixture_depth` tracks `@pytest.fixture`
/// nesting, so `no-inline-patch` allows a patch there and flags one in a test body.
struct LintVisitor<'a> {
    file: &'a Path,
    source: &'a str,
    fixture_depth: usize,
    /// The dist's own top-level package, or `None` when undiscoverable.
    first_party: Option<&'a str>,
    /// Local name → the dotted module path its import binds, for object patch targets.
    imports: HashMap<String, String>,
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

    /// Run the parameter lint, and return whether this function is a fixture.
    fn enter_function(&mut self, args: &Arguments, decorators: &[Expr], range: TextRange) -> bool {
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
        // A fixture is the right place for a patch; a test body is not.
        if is_patch_call(&node) && self.fixture_depth == 0 {
            self.report(
                node.range,
                "no-inline-patch",
                "patch is called inline in a test body; move it into a `pytest.fixture`",
            );
        }
        // Both target rules fire regardless of fixture depth — a config constant is usually
        // patched in one — and only on a statically resolved target.
        if let Some(target) = patch_target(&node, &self.imports) {
            if patches_constant(&target) {
                self.report(node.range, "no-constant-patch", CONSTANT_PATCH_MSG);
            }
            if let Some(pkg) = self.first_party {
                if patches_first_party(&target, pkg) {
                    self.report(node.range, "no-first-party-patch", FIRST_PARTY_PATCH_MSG);
                }
            }
        }
        if is_environ_mutation_call(&node) {
            self.report(node.range, "no-environ-mutation", ENVIRON_MUTATION_MSG);
        }
        self.generic_visit_expr_call(node);
    }

    fn visit_stmt_import(&mut self, node: StmtImport) {
        for alias in &node.names {
            match &alias.asname {
                // `import X.Y as A` binds `A`; a plain `import X.Y` binds only the head `X`.
                Some(asname) => {
                    self.imports
                        .insert(asname.to_string(), alias.name.to_string());
                }
                None => {
                    let head = import_head(alias.name.as_str());
                    self.imports.insert(head.to_string(), head.to_string());
                }
            }
        }
        self.generic_visit_stmt_import(node);
    }

    fn visit_stmt_import_from(&mut self, node: StmtImportFrom) {
        // A relative import names no absolute module, so its bindings resolve nothing.
        if relative_level(&node) == 0 {
            if let Some(module) = &node.module {
                for alias in &node.names {
                    let bound = alias.asname.as_ref().unwrap_or(&alias.name);
                    self.imports
                        .insert(bound.to_string(), format!("{module}.{}", alias.name));
                }
            }
        }
        self.generic_visit_stmt_import_from(node);
    }

    // The generated `generic_visit_withitem` is a no-op, so a `with patch(...)`
    // context expression is never walked unless we descend into it here.
    fn visit_withitem(&mut self, node: WithItem) {
        self.visit_expr(node.context_expr);
        if let Some(optional_vars) = node.optional_vars {
            self.visit_expr(*optional_vars);
        }
    }

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

/// `true` for an `@pytest.fixture` / `@fixture` decorator, called or bare.
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

/// The three call shapes of `unittest.mock.patch`, which name their target differently.
enum PatchForm {
    /// `patch("pkg.mod.attr")` — the target is the string-literal first argument.
    Target,
    /// `patch.object(base, "attr")` — the target is `base`'s module plus the attribute.
    Object,
    /// `patch.dict(base_or_string, ...)` — the target is the dict itself.
    Dict,
}

/// The form of a `patch(...)` / `patch.object(...)` / `patch.dict(...)` call, plain or
/// reached through a module (`mock.patch(...)`, `unittest.mock.patch`). `None` otherwise.
fn patch_form(call: &ExprCall) -> Option<PatchForm> {
    match call.func.as_ref() {
        Expr::Name(name) if name.id.as_str() == "patch" => Some(PatchForm::Target),
        Expr::Attribute(attr) => match attr.attr.as_str() {
            "patch" => Some(PatchForm::Target),
            "object" if attr_base_is_patch(attr.value.as_ref()) => Some(PatchForm::Object),
            "dict" if attr_base_is_patch(attr.value.as_ref()) => Some(PatchForm::Dict),
            _ => None,
        },
        _ => None,
    }
}

/// `true` for any [`PatchForm`] call.
fn is_patch_call(call: &ExprCall) -> bool {
    patch_form(call).is_some()
}

/// `true` when an attribute's base resolves to `patch` — a `patch.object` receiver.
fn attr_base_is_patch(expr: &Expr) -> bool {
    match expr {
        Expr::Name(name) => name.id.as_str() == "patch",
        Expr::Attribute(attr) => attr.attr.as_str() == "patch",
        _ => false,
    }
}

const CONSTANT_PATCH_MSG: &str = "patches a module-global config constant; inject config explicitly (a consumer that did `from pkg import CONSTANT` snapshots the value at import time and ignores the patch)";

const FIRST_PARTY_PATCH_MSG: &str = "patches a first-party target; an integration test must run first-party code for real — only third-party packages and effectful stdlib may be patched";

/// The string-literal first argument of a `patch(...)` call, the dotted target. `None` for
/// a non-literal argument, which can't be classified deterministically.
fn patch_string_target(call: &ExprCall) -> Option<&str> {
    string_arg(call, 0)
}

/// The string literal at argument position `index` of a call, if that is what sits there.
fn string_arg(call: &ExprCall, index: usize) -> Option<&str> {
    if let Some(Expr::Constant(constant)) = call.args.get(index) {
        if let Constant::Str(value) = &constant.value {
            return Some(value.as_str());
        }
    }
    None
}

/// The dotted segments of a plain attribute chain (`myproject.ledger` → `["myproject",
/// "ledger"]`). `None` for a chain rooted in anything but a name.
fn attr_chain_segments(expr: &Expr) -> Option<Vec<&str>> {
    match expr {
        Expr::Name(name) => Some(vec![name.id.as_str()]),
        Expr::Attribute(attr) => {
            let mut segments = attr_chain_segments(attr.value.as_ref())?;
            segments.push(attr.attr.as_str());
            Some(segments)
        }
        _ => None,
    }
}

/// The dotted module path an object target names, its head replaced by the module its import
/// binds (`ledger` → `myproject.ledger` after `from myproject import ledger`). `None` when no
/// import binds the head — a local name has no statically known module.
fn resolve_object_target(expr: &Expr, imports: &HashMap<String, String>) -> Option<String> {
    let segments = attr_chain_segments(expr)?;
    let (head, rest) = segments.split_first()?;
    let mut target = imports.get(*head)?.clone();
    for segment in rest {
        target.push('.');
        target.push_str(segment);
    }
    Some(target)
}

/// The dotted target a patch call names, resolved statically: the string literal for
/// `patch(...)` (and a string-target `patch.dict`), the import-resolved first argument for
/// the object forms. `None` when the target resists static resolution — nothing fires.
fn patch_target(call: &ExprCall, imports: &HashMap<String, String>) -> Option<String> {
    match patch_form(call)? {
        PatchForm::Target => patch_string_target(call).map(str::to_owned),
        PatchForm::Dict => patch_string_target(call)
            .map(str::to_owned)
            .or_else(|| resolve_object_target(call.args.first()?, imports)),
        PatchForm::Object => {
            let base = resolve_object_target(call.args.first()?, imports)?;
            Some(match string_arg(call, 1) {
                Some(attr) => format!("{base}.{attr}"),
                None => base,
            })
        }
    }
}

/// `true` when a patch target names an UPPER_CASE constant (`"pkg.cfg.CACHE_DIR"`).
fn patches_constant(target: &str) -> bool {
    target.rsplit('.').next().is_some_and(is_upper_constant)
}

/// `true` when patch `target`'s head segment names the first-party package `pkg`.
fn patches_first_party(target: &str, pkg: &str) -> bool {
    target
        .split('.')
        .next()
        .is_some_and(|head| !head.is_empty() && head == pkg)
}

/// `true` for an ALL-CAPS name: uppercase letters, digits, underscores, one letter minimum.
fn is_upper_constant(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        && name.chars().any(|c| c.is_ascii_uppercase())
}

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

/// `true` for `os.environ[...]`, the form used as an assignment or `del` target.
fn is_os_environ_subscript(expr: &Expr) -> bool {
    matches!(expr, Expr::Subscript(sub) if is_os_environ(sub.value.as_ref()))
}

/// `true` for a mutating method call on `os.environ`, like `os.environ.update(...)`.
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

/// The dist's own top-level import package: the nearest `pyproject.toml`'s `[project].name`,
/// [normalized](normalize_dist_name). The walk up stops at a `.git` boundary so it can't
/// escape into an unrelated project, and `None` means nothing is flagged rather than guessed.
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

/// A distribution name as its import package name, PEP 503-flavoured: `My-Project` →
/// `my_project`.
fn normalize_dist_name(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace(['-', '.'], "_")
}

fn collect_python_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    is_match: fn(&Path) -> bool,
) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_python_files(&path, out, is_match)?;
        } else if is_match(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` for a file the integration lints scan: `*_test.py` or `conftest.py`. A legacy
/// `test_*.py` is ordinary source.
fn is_python_test_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name == "conftest.py" || name.ends_with("_test.py")
}

/// `true` for a colocated unit test: `*_test.py`. A legacy `test_*.py` is ordinary source,
/// and `conftest.py` holds fixtures rather than a unit.
fn is_python_unit_test_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name.ends_with("_test.py")
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
            let path = self.0.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            std::fs::write(path, contents).unwrap();
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
        let stmt = suite.into_iter().next().expect("one statement");
        (*stmt.expect_expr_stmt().value).expect_call_expr()
    }

    #[test]
    fn patch_target_only_reads_string_literals_for_the_string_form() {
        let imports = HashMap::new();
        let str_call = parse_call("patch(\"pkg.mod.attr\")\n");
        assert_eq!(
            patch_target(&str_call, &imports).as_deref(),
            Some("pkg.mod.attr")
        );
        // A name in `patch(...)` holds a string, which static resolution cannot read.
        let name_call = parse_call("patch(target)\n");
        assert_eq!(patch_target(&name_call, &imports), None);
        let int_call = parse_call("patch(42)\n");
        assert_eq!(patch_target(&int_call, &imports), None);
        let empty_call = parse_call("patch()\n");
        assert_eq!(patch_target(&empty_call, &imports), None);
    }

    /// An import map binding the names the object-form snippets use.
    fn object_form_imports() -> HashMap<String, String> {
        HashMap::from([
            ("ledger".to_string(), "myproject.ledger".to_string()),
            ("myproject".to_string(), "myproject".to_string()),
            ("cfg".to_string(), "myproject.cfg".to_string()),
        ])
    }

    #[test]
    fn patch_target_resolves_object_forms_through_imports() {
        let imports = object_form_imports();
        let imported_name = parse_call("patch.object(ledger, \"record\")\n");
        assert_eq!(
            patch_target(&imported_name, &imports).as_deref(),
            Some("myproject.ledger.record")
        );
        let dotted_module = parse_call("patch.object(myproject.ledger, \"record\")\n");
        assert_eq!(
            patch_target(&dotted_module, &imports).as_deref(),
            Some("myproject.ledger.record")
        );
        // A non-literal attribute still names the base module, enough for the first-party rule.
        let name_attr = parse_call("patch.object(ledger, attr)\n");
        assert_eq!(
            patch_target(&name_attr, &imports).as_deref(),
            Some("myproject.ledger")
        );
        let dict_object = parse_call("patch.dict(cfg.SETTINGS, {})\n");
        assert_eq!(
            patch_target(&dict_object, &imports).as_deref(),
            Some("myproject.cfg.SETTINGS")
        );
        let dict_string = parse_call("patch.dict(\"pkg.cfg.FLAGS\", {})\n");
        assert_eq!(
            patch_target(&dict_string, &imports).as_deref(),
            Some("pkg.cfg.FLAGS")
        );
    }

    #[test]
    fn patch_target_declines_a_base_bound_by_no_import() {
        let imports = object_form_imports();
        let call_base = parse_call("patch.object(get_mod(), \"x\")\n");
        assert_eq!(patch_target(&call_base, &imports), None);
        let unbound_name = parse_call("patch.object(client, \"send\")\n");
        assert_eq!(patch_target(&unbound_name, &imports), None);
        let empty = parse_call("patch.object()\n");
        assert_eq!(patch_target(&empty, &imports), None);
    }

    /// The imports a [`LintVisitor`] records for `src`.
    fn collect_imports(src: &str) -> HashMap<String, String> {
        let suite = ast::Suite::parse(src, "t.py").expect("snippet should parse");
        let mut visitor = LintVisitor {
            file: Path::new("t.py"),
            source: src,
            fixture_depth: 0,
            first_party: None,
            imports: HashMap::new(),
            violations: Vec::new(),
        };
        for stmt in suite {
            visitor.visit_stmt(stmt);
        }
        visitor.imports
    }

    #[test]
    fn lint_visitor_binds_imports_to_their_modules() {
        let imports = collect_imports(
            "import myproject.ledger\n\
             import myproject.config as cfg\n\
             from myproject import ledger\n\
             from myproject import charge as ch\n\
             from . import rel\n",
        );
        assert_eq!(
            imports.get("myproject").map(String::as_str),
            Some("myproject")
        );
        assert_eq!(
            imports.get("cfg").map(String::as_str),
            Some("myproject.config")
        );
        assert_eq!(
            imports.get("ledger").map(String::as_str),
            Some("myproject.ledger")
        );
        assert_eq!(
            imports.get("ch").map(String::as_str),
            Some("myproject.charge")
        );
        assert_eq!(imports.get("rel"), None);
    }

    /// Build a `from <source> import <symbols>` record (`source: None` → relative).
    fn from_import(source: Option<&str>, symbols: &[&str]) -> ImportRecord {
        ImportRecord {
            display: source.unwrap_or(".rel").to_string(),
            line: 1,
            is_uut: false,
            symbols: symbols.iter().map(|s| (*s).to_string()).collect(),
            source: source.map(str::to_string),
            module: None,
        }
    }

    fn targets(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn is_mocked_requires_every_symbol_at_the_import_module() {
        let rec = from_import(Some("pkg.ledger"), &["record", "erase"]);
        // Only `record` patched → the un-mocked `erase` leaves the import un-mocked.
        assert!(!rec.is_mocked(&targets(&["pkg.ledger.record"])));
        assert!(rec.is_mocked(&targets(&["pkg.ledger.record", "pkg.ledger.erase"])));
    }

    #[test]
    fn is_mocked_rejects_a_last_segment_match_in_another_module() {
        let rec = from_import(Some("pkg.ledger"), &["record"]);
        // Same last segment, different module → not mocked.
        assert!(!rec.is_mocked(&targets(&["otherpkg.unrelated.record"])));
        let dumps = from_import(Some("pkg.formatter"), &["dumps"]);
        assert!(!dumps.is_mocked(&targets(&["json.dumps"])));
        assert!(rec.is_mocked(&targets(&["pkg.ledger.record"])));
    }

    #[test]
    fn is_mocked_relative_import_accepts_a_last_segment_match() {
        // A relative import has no module to compare, so a last-segment match is accepted.
        let rec = from_import(None, &["record"]);
        assert!(rec.is_mocked(&targets(&["pkg.ledger.record"])));
        assert!(!rec.is_mocked(&targets(&["pkg.ledger.other"])));
    }

    #[test]
    fn is_mocked_module_import_matches_a_patch_reaching_in() {
        let rec = ImportRecord {
            display: "pkg.db".to_string(),
            line: 1,
            is_uut: false,
            symbols: Vec::new(),
            source: None,
            module: Some("pkg.db".to_string()),
        };
        assert!(rec.is_mocked(&targets(&["pkg.db.connect"])));
        assert!(rec.is_mocked(&targets(&["pkg.db"])));
        assert!(!rec.is_mocked(&targets(&["pkg.other.connect"])));
        let empty = from_import(Some("pkg.mod"), &[]);
        assert!(!empty.is_mocked(&targets(&["pkg.mod.thing"])));
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
        assert_eq!(first_party_package(&tree.0).as_deref(), Some("my_project"));
    }

    #[test]
    fn first_party_package_is_none_without_a_project_name() {
        let tree = TempDir::new();
        tree.write("pyproject.toml", "[build-system]\nrequires = []\n");
        tree.write(".git", "");
        assert_eq!(first_party_package(&tree.0), None);
    }

    #[test]
    fn first_party_package_is_none_when_absent() {
        let tree = TempDir::new();
        assert_eq!(first_party_package(&tree.0), None);
    }

    /// The displays of the imports `source` leaves un-mocked.
    fn unmocked(base: &str, first_party: &str, source: &str) -> Vec<String> {
        let suite = ast::Suite::parse(source, "t.py").expect("snippet should parse");
        let mut visitor = UnitIsolationVisitor {
            source,
            first_party,
            base,
            type_checking_depth: 0,
            imports: Vec::new(),
            patch_targets: Vec::new(),
        };
        for stmt in suite {
            visitor.visit_stmt(stmt);
        }
        visitor
            .imports
            .iter()
            .filter(|i| !i.is_uut && !i.is_mocked(&visitor.patch_targets))
            .map(|i| i.display.clone())
            .collect()
    }

    #[test]
    fn import_head_and_last_segment() {
        assert_eq!(import_head("myproject.db.conn"), "myproject");
        assert_eq!(import_head("requests"), "requests");
        assert_eq!(last_segment("myproject.db.conn"), "conn");
        assert_eq!(last_segment("widget"), "widget");
    }

    #[test]
    fn unit_under_test_base_strips_test_suffix() {
        assert_eq!(
            unit_under_test_base(Path::new("pkg/widget_test.py")),
            "widget"
        );
        // Only `*_test.py` reaches here, so a legacy `test_*.py` keeps its prefix.
        assert_eq!(
            unit_under_test_base(Path::new("test_widget.py")),
            "test_widget"
        );
        assert_eq!(unit_under_test_base(Path::new("plain.py")), "plain");
    }

    #[test]
    fn recognizes_python_unit_test_files() {
        assert!(is_python_unit_test_file(Path::new("widget_test.py")));
        assert!(is_python_unit_test_file(Path::new("pkg/widget_test.py")));
        assert!(!is_python_unit_test_file(Path::new("test_widget.py")));
        assert!(!is_python_unit_test_file(Path::new("conftest.py")));
        assert!(!is_python_unit_test_file(Path::new("widget.py")));
    }

    #[test]
    fn visitor_flags_first_party_and_external_collaborators() {
        // The UUT is left alone; the first-party and third-party imports are flagged.
        let found = unmocked(
            "widget",
            "myproject",
            "from myproject.widget import build\n\
             from myproject.ledger import record\n\
             import requests\n",
        );
        assert_eq!(
            found,
            vec!["myproject.ledger".to_string(), "requests".to_string()]
        );
    }

    #[test]
    fn visitor_clears_a_mocked_collaborator() {
        let found = unmocked(
            "widget",
            "myproject",
            "from myproject.ledger import record\npatch(\"myproject.ledger.record\")\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn visitor_flags_a_wrong_module_patch() {
        // A patch sharing only the last segment names a different module, so `record`
        // stays an un-mocked collaborator.
        let found = unmocked(
            "widget",
            "myproject",
            "from myproject.ledger import record\npatch(\"otherpkg.unrelated.record\")\n",
        );
        assert_eq!(found, vec!["myproject.ledger".to_string()]);
    }

    #[test]
    fn visitor_flags_a_partly_mocked_multi_symbol_import() {
        // Patching only `record` leaves the sibling `erase` a real collaborator.
        let found = unmocked(
            "widget",
            "myproject",
            "from myproject.ledger import record, erase\npatch(\"myproject.ledger.record\")\n",
        );
        assert_eq!(found, vec!["myproject.ledger".to_string()]);
        let both = unmocked(
            "widget",
            "myproject",
            "from myproject.ledger import record, erase\n\
             patch(\"myproject.ledger.record\")\npatch(\"myproject.ledger.erase\")\n",
        );
        assert!(both.is_empty(), "got: {both:?}");
    }

    #[test]
    fn visitor_handles_module_and_relative_imports() {
        assert_eq!(
            unmocked("widget", "myproject", "import myproject.db\n"),
            vec!["myproject.db".to_string()]
        );
        assert!(unmocked(
            "widget",
            "myproject",
            "import myproject.db\npatch(\"myproject.db.connect\")\n"
        )
        .is_empty());
        assert_eq!(
            unmocked("widget", "myproject", "from .ledger import record\n"),
            vec![".ledger".to_string()]
        );
        assert_eq!(
            unmocked(
                "widget",
                "myproject",
                "from . import ledger\nfrom . import widget\n"
            ),
            vec![".ledger".to_string()]
        );
    }

    #[test]
    fn visitor_treats_barrel_reexport_import_as_the_unit_under_test() {
        // A bare `from . import …` names the package's own re-export surface, the SUT.
        assert!(unmocked(
            "__init__",
            "myproject",
            "from . import Thing, __all__, __version__\n"
        )
        .is_empty());
        // Reaching around the barrel into a sibling module is still a collaborator.
        assert_eq!(
            unmocked("__init__", "myproject", "from .core import Thing\n"),
            vec![".core".to_string()]
        );
        // `from .. import x` resolves to the parent package, not the SUT file.
        assert_eq!(
            unmocked("__init__", "myproject", "from .. import sibling\n"),
            vec!["..sibling".to_string()]
        );
        // The barrel shortcut is scoped to the `__init__` base.
        assert_eq!(
            unmocked("widget", "myproject", "from . import ledger\n"),
            vec![".ledger".to_string()]
        );
    }

    #[test]
    fn visitor_skips_type_checking_imports() {
        // A TYPE_CHECKING import is type-only; the runtime `else` import is still seen.
        let found = unmocked(
            "widget",
            "myproject",
            "if TYPE_CHECKING:\n    from myproject.models import Widget\nelse:\n    from myproject.ledger import record\n",
        );
        assert_eq!(found, vec!["myproject.ledger".to_string()]);
    }

    #[test]
    fn is_checked_import_classifies_origins() {
        assert!(is_checked_import("myproject", "myproject")); // first-party
        assert!(!is_checked_import("pytest", "myproject")); // test framework
        assert!(!is_checked_import("_pytest", "myproject"));
        assert!(is_checked_import("subprocess", "myproject")); // effectful stdlib
        assert!(is_checked_import("socket", "myproject"));
        assert!(!is_checked_import("json", "myproject")); // pure stdlib
        assert!(!is_checked_import("dataclasses", "myproject"));
        assert!(is_checked_import("requests", "myproject")); // third-party
        assert!(is_checked_import("stripe", "myproject"));
        // A dual-nature head stays pure — the patch convention catches it, not the import.
        assert!(!is_checked_import("os", "myproject"));
        assert!(!is_checked_import("pathlib", "myproject"));
        assert!(!is_checked_import("datetime", "myproject"));
    }

    #[test]
    fn is_checked_import_classifies_private_stdlib_as_stdlib() {
        assert!(!is_checked_import("__future__", "myproject"));
        assert!(!is_checked_import("_thread", "myproject"));
        assert!(!is_checked_import("_socket", "myproject"));
        assert!(!is_checked_import("_ast", "myproject"));
        assert!(!is_checked_import("_collections_abc", "myproject"));
        assert!(is_checked_import("_stripe", "myproject")); // third-party
    }

    #[test]
    fn visitor_flags_external_collaborators() {
        let found = unmocked(
            "widget",
            "myproject",
            "import requests\nimport subprocess\nimport json\nimport pytest\n",
        );
        assert_eq!(found.len(), 2, "got: {found:?}");
        assert!(found.contains(&"requests".to_string()));
        assert!(found.contains(&"subprocess".to_string()));
    }

    #[test]
    fn visitor_type_checking_variants_and_plain_if() {
        // The attribute form guards type-only imports too.
        assert!(unmocked(
            "widget",
            "myproject",
            "if typing.TYPE_CHECKING:\n    from myproject.models import W\n    import myproject.db\n"
        )
        .is_empty());
        // A plain `if` is walked normally; its import is still a collaborator.
        assert_eq!(
            unmocked(
                "widget",
                "myproject",
                "if ready == 1:\n    from myproject.ledger import record\n"
            ),
            vec!["myproject.ledger".to_string()]
        );
    }

    #[test]
    fn find_unit_isolation_without_pyproject_reports_nothing() {
        let tree = TempDir::new();
        tree.write("widget_test.py", "from myproject.ledger import record\n");
        tree.write(".git", "");
        assert!(find_unit_isolation_violations(&tree.0)
            .expect("a readable tree should succeed")
            .is_empty());
    }

    #[test]
    fn find_unit_isolation_walks_subdirs_and_flags() {
        let tree = TempDir::new();
        tree.write("pyproject.toml", "[project]\nname = \"myproject\"\n");
        tree.write("pkg/thing_test.py", "from myproject.ledger import record\n");
        let found =
            find_unit_isolation_violations(&tree.0).expect("a readable tree should succeed");
        assert_eq!(found.len(), 1, "got: {found:?}");
        assert_eq!(found[0].rule, "unmocked-collaborator");
        assert!(found[0].message.contains("myproject.ledger"));
    }

    #[test]
    fn recognizes_python_test_files() {
        assert!(is_python_test_file(Path::new("widget_test.py")));
        assert!(is_python_test_file(Path::new("pkg/widget_test.py")));
        assert!(is_python_test_file(Path::new("conftest.py")));
        assert!(!is_python_test_file(Path::new("test_widget.py")));
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

    /// The rules the suite lint reports for `source`, in report order.
    fn lint_rules(source: &str) -> Vec<&'static str> {
        let suite = ast::Suite::parse(source, "t.py").expect("snippet should parse");
        let mut visitor = LintVisitor {
            file: Path::new("t.py"),
            source,
            fixture_depth: 0,
            first_party: Some("myproject"),
            imports: HashMap::new(),
            violations: Vec::new(),
        };
        for stmt in suite {
            visitor.visit_stmt(stmt);
        }
        visitor.violations.iter().map(|v| v.rule).collect()
    }

    #[test]
    fn an_async_fixture_shelters_a_patch_that_an_async_test_does_not() {
        assert!(
            lint_rules("@pytest.fixture\nasync def client():\n    patch(\"pkg.mod.attr\")\n")
                .is_empty()
        );
        assert_eq!(
            lint_rules("async def widget_test():\n    patch(\"pkg.mod.attr\")\n"),
            vec!["no-inline-patch"]
        );
        assert_eq!(
            lint_rules("async def widget_test(monkeypatch):\n    pass\n"),
            vec!["no-monkeypatch"]
        );
    }

    #[test]
    fn an_augmented_assignment_to_environ_is_a_mutation() {
        assert_eq!(
            lint_rules("def widget_test():\n    os.environ[\"PATH\"] += \":/x\"\n"),
            vec!["no-environ-mutation"]
        );
        assert!(lint_rules("def widget_test():\n    total += 1\n").is_empty());
    }

    #[test]
    fn a_fixture_decorator_is_a_bare_name_or_an_attribute() {
        assert!(
            lint_rules("@fixture\ndef client():\n    patch(\"pkg.mod.attr\")\n").is_empty(),
            "a bare `@fixture` shelters the patch"
        );
        assert_eq!(
            lint_rules("@registry[\"fixture\"]\ndef client():\n    patch(\"pkg.mod.attr\")\n"),
            vec!["no-inline-patch"],
            "a subscripted decorator is not a fixture"
        );
    }

    #[test]
    fn patch_object_is_recognized_only_through_a_patch_receiver() {
        assert_eq!(
            lint_rules("def widget_test():\n    mock.patch.object(svc, \"send\")\n"),
            vec!["no-inline-patch"]
        );
        assert!(
            lint_rules("def widget_test():\n    helpers[0].object(svc, \"send\")\n").is_empty(),
            "a subscripted receiver is not `patch`"
        );
        assert!(
            lint_rules("def widget_test():\n    helpers[0](\"pkg.mod.attr\")\n").is_empty(),
            "a subscripted callee is not a patch call"
        );
    }

    #[test]
    fn find_suite_without_a_tests_directory_reports_nothing() {
        let tree = TempDir::new();
        tree.write("pyproject.toml", "[project]\nname = \"myproject\"\n");
        assert!(find_suite_violations(&tree.0)
            .expect("a readable tree should succeed")
            .is_empty());
    }

    #[test]
    fn find_unit_isolation_without_a_project_name_reports_nothing() {
        let tree = TempDir::new();
        tree.write("pyproject.toml", "[build-system]\nrequires = []\n");
        tree.write("widget_test.py", "from myproject.ledger import record\n");
        assert!(find_unit_isolation_violations(&tree.0)
            .expect("a readable tree should succeed")
            .is_empty());
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
