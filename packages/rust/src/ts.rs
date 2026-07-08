//! TypeScript isolation analysis, parsed with `oxc`.
//!
//! This is the TypeScript counterpart to the Python [`crate::lint`] module. In
//! the *integration direction*, an integration test runs first-party code for
//! real, so it may mock third-party packages and Node built-ins but **never** a
//! first-party module.
//!
//! Detection is AST-based — each `*.test.{ts,tsx,mts,cts}` file is parsed with
//! `oxc_parser` and walked for `vi.mock()` / `vi.doMock()` calls whose target
//! specifier is first-party. The specifier [`classify`]-ication (first-party /
//! Node-builtin / third-party) is the shared foundation the unit-direction
//! slices build on.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use oxc::allocator::Allocator;
use oxc::ast::ast::{Argument, CallExpression, Expression, ImportDeclaration, ImportOrExportKind};
use oxc::ast_visit::{walk, Visit};
use oxc::parser::Parser;
use oxc::span::{SourceType, Span};

use crate::lint::Violation;

/// Where a module specifier resolves, for isolation purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// A relative or absolute path (`./x`, `../x`, `/abs`) — first-party code.
    FirstParty,
    /// A Node.js built-in (`node:fs`, `fs`, `fs/promises`, `path`, …).
    Builtin,
    /// Any other bare specifier — a third-party package (`lodash`, `@scope/x`).
    ThirdParty,
}

/// Classify a module specifier as first-party, Node-builtin, or third-party.
///
/// Deterministic and resolution-free — the bright-line rule the README's
/// isolation checks rest on:
/// - a **relative or absolute** path (`./`, `../`, `/`) is first-party;
/// - a `node:`-prefixed specifier, or one whose first path segment is a known
///   Node built-in (so `fs` and `fs/promises` both match), is a built-in;
/// - every other (bare) specifier is a third-party package.
pub fn classify(specifier: &str) -> Origin {
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return Origin::FirstParty;
    }
    if specifier.starts_with("node:") || is_node_builtin(specifier) {
        return Origin::Builtin;
    }
    Origin::ThirdParty
}

/// `true` when `specifier`'s first path segment is a Node.js built-in module —
/// so a subpath export like `fs/promises` matches on its `fs` head.
fn is_node_builtin(specifier: &str) -> bool {
    let head = specifier.split('/').next().unwrap_or(specifier);
    NODE_BUILTINS.contains(&head)
}

/// The Node.js built-in module names (the stable set). The explicit `node:`
/// prefix is handled separately in [`classify`], so future built-ins stay
/// recognized when written `node:<name>`.
const NODE_BUILTINS: &[&str] = &[
    "assert",
    "async_hooks",
    "buffer",
    "child_process",
    "cluster",
    "console",
    "constants",
    "crypto",
    "dgram",
    "diagnostics_channel",
    "dns",
    "domain",
    "events",
    "fs",
    "http",
    "http2",
    "https",
    "inspector",
    "module",
    "net",
    "os",
    "path",
    "perf_hooks",
    "process",
    "punycode",
    "querystring",
    "readline",
    "repl",
    "stream",
    "string_decoder",
    "sys",
    "timers",
    "tls",
    "trace_events",
    "tty",
    "url",
    "util",
    "v8",
    "vm",
    "wasi",
    "worker_threads",
    "zlib",
];

/// Scan the TypeScript test files under `root` and return every
/// integration-isolation violation, sorted by `(file, line)` for deterministic
/// output.
///
/// A *TypeScript test file* is `*.test.{ts,tsx,mts,cts}`. Each is parsed and
/// walked; a file that cannot be read or parsed is an error.
pub fn find_integration_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_ts_test_files(root, &mut files)?;
    files.sort();

    let mut violations = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading test file `{}`", file.display()))?;
        violations.extend(integration_violations_in(file, &source)?);
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// Scan the unit test files under `root` and return every isolation violation —
/// a runtime import that isn't `vi.mock()`-ed — sorted by `(file, line)`.
/// The TypeScript arm of `unit lint`
/// ([`crate::isolation::Language::TypeScript`]).
///
/// A *TypeScript unit test* is `*.test.{ts,tsx,mts,cts}`. Each is parsed and
/// walked; a file that cannot be read or parsed is an error.
pub fn find_unit_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_ts_test_files(root, &mut files)?;
    files.sort();

    let mut violations = Vec::new();
    for file in &files {
        let source = std::fs::read_to_string(file)
            .with_context(|| format!("reading test file `{}`", file.display()))?;
        violations.extend(unit_violations_in(file, &source)?);
    }

    violations.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));
    Ok(violations)
}

/// Parse one unit test file and collect its `unmocked-collaborator` violations:
/// every runtime import that isn't the unit under test, the test runner, or
/// `vi.mock()`-ed.
fn unit_violations_in(file: &Path, source: &str) -> Result<Vec<Violation>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file).map_err(|err| {
        anyhow!(
            "unsupported TypeScript extension `{}`: {err}",
            file.display()
        )
    })?;
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked || !ret.diagnostics.is_empty() {
        let detail = ret
            .diagnostics
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        bail!("parsing `{}` failed: {detail}", file.display());
    }

    let mut collector = UnitCollector {
        source,
        imports: Vec::new(),
        mocked: BTreeSet::new(),
        untyped: Vec::new(),
    };
    collector.visit_program(&ret.program);

    let unit = unit_under_test_specifier(file);
    let mut violations = Vec::new();
    for (spec, line) in &collector.imports {
        if is_unit_under_test(spec, &unit)
            || is_test_runner(spec)
            || collector.mocked.contains(spec)
        {
            continue;
        }
        violations.push(Violation {
            file: file.to_path_buf(),
            line: *line,
            rule: "unmocked-collaborator",
            message: format!(
                "unit test imports `{spec}` without mocking it — a unit test isolates the \
                 unit under test, so every collaborator must be `vi.mock()`-ed"
            ),
        });
    }
    for (spec, line) in &collector.untyped {
        violations.push(Violation {
            file: file.to_path_buf(),
            line: *line,
            rule: "untyped-mock",
            message: format!(
                "`vi.mock('{spec}', …)` has an untyped factory — anchor it to the real module \
                 with `vi.importActual<typeof import('{spec}')>()` so the double can't drift \
                 from the source"
            ),
        });
    }
    violations.sort_by_key(|v| v.line);
    Ok(violations)
}

/// Collects a unit test's runtime imports (specifier + line), its `vi.mock()`
/// targets, and any `vi.mock()` with an untyped factory in one AST pass.
struct UnitCollector<'s> {
    source: &'s str,
    imports: Vec<(String, usize)>,
    mocked: BTreeSet<String>,
    untyped: Vec<(String, usize)>,
}

impl<'a> Visit<'a> for UnitCollector<'_> {
    fn visit_import_declaration(&mut self, decl: &ImportDeclaration<'a>) {
        // `import type …` is erased at compile time — not a runtime dependency.
        if matches!(decl.import_kind, ImportOrExportKind::Type) {
            return;
        }
        self.imports.push((
            decl.source.value.to_string(),
            line_of(self.source, decl.span.start),
        ));
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(spec) = vi_mock_target(call) {
            // A factory *function* (2nd arg) that doesn't anchor to the real
            // module's type via `vi.importActual<…>()` lets the double drift from
            // the source. The 2nd arg is only a factory when it's a function:
            // a bare `vi.mock(spec)` is an auto-mock (typed from the real module),
            // and so is the options form `vi.mock(spec, { spy: true })`, which spies
            // on the real module and can't drift — neither is flagged.
            if let Some(factory) = call.arguments.get(1) {
                if is_factory(factory) && !factory_is_typed(factory) {
                    self.untyped
                        .push((spec.clone(), line_of(self.source, call.span.start)));
                }
            }
            self.mocked.insert(spec);
        }
        walk::walk_call_expression(self, call);
    }
}

/// The unit-under-test specifier for a test file: `pkg/widget.test.ts` → `./widget`.
fn unit_under_test_specifier(file: &Path) -> String {
    let name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let stem = name.split(".test.").next().unwrap_or(name);
    format!("./{stem}")
}

/// `true` when `spec` resolves to the unit under test, ignoring an explicit
/// module extension (`./widget` and `./widget.js` both match `./widget`).
fn is_unit_under_test(spec: &str, unit: &str) -> bool {
    strip_module_ext(spec) == unit
}

/// `spec` without a trailing JS/TS module extension.
fn strip_module_ext(spec: &str) -> &str {
    for ext in [".js", ".mjs", ".cjs", ".jsx", ".ts", ".mts", ".cts", ".tsx"] {
        if let Some(base) = spec.strip_suffix(ext) {
            return base;
        }
    }
    spec
}

/// `true` for the Vitest test runner itself (`vitest`, `vitest/*`, `@vitest/*`) —
/// the harness, never a collaborator to mock.
fn is_test_runner(spec: &str) -> bool {
    spec == "vitest" || spec.starts_with("vitest/") || spec.starts_with("@vitest/")
}

/// `true` when a `vi.mock` second argument is a factory *function* — an arrow or
/// `function` expression. Vitest's other 2nd-arg form is an options object
/// (`vi.mock(spec, { spy: true })`), which is **not** a factory: it spies on the
/// real module, so the double can't drift, exactly like a bare `vi.mock(spec)`
/// auto-mock. Only a function factory can return a hand-built double that
/// needs a `vi.importActual<…>` type anchor.
fn is_factory(arg: &Argument) -> bool {
    matches!(
        arg.as_expression(),
        Some(Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_))
    )
}

/// `true` when a `vi.mock` factory anchors to the real module's type — i.e. its
/// body contains a `vi.importActual<…>()` call carrying a type argument.
/// The conventional form is `vi.importActual<typeof import('<spec>')>()`.
fn factory_is_typed(factory: &Argument) -> bool {
    let mut finder = ImportActualFinder { typed: false };
    finder.visit_argument(factory);
    finder.typed
}

/// Walks a `vi.mock` factory looking for a typed `vi.importActual<…>()` call.
struct ImportActualFinder {
    typed: bool,
}

impl<'a> Visit<'a> for ImportActualFinder {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_typed_import_actual(call) {
            self.typed = true;
        }
        walk::walk_call_expression(self, call);
    }
}

/// `true` for `vi.importActual<…>(…)` — a call to `vi.importActual` that carries a
/// type argument (an untyped `vi.importActual(…)` returns `unknown`).
fn is_typed_import_actual(call: &CallExpression) -> bool {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return false;
    };
    let is_vi = matches!(&member.object, Expression::Identifier(id) if id.name == "vi");
    is_vi && member.property.name.as_str() == "importActual" && call.type_arguments.is_some()
}

/// Parse one TypeScript test file and collect its `no-first-party-mock`
/// violations. A parse failure is an error — a malformed test file is never a
/// silent pass.
fn integration_violations_in(file: &Path, source: &str) -> Result<Vec<Violation>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(file).map_err(|err| {
        anyhow!(
            "unsupported TypeScript extension `{}`: {err}",
            file.display()
        )
    })?;
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked || !ret.diagnostics.is_empty() {
        let detail = ret
            .diagnostics
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        bail!("parsing `{}` failed: {detail}", file.display());
    }

    let mut visitor = MockVisitor {
        file,
        source,
        violations: Vec::new(),
    };
    visitor.visit_program(&ret.program);
    Ok(visitor.violations)
}

/// Walks one parsed test file, flagging every `vi.mock()` / `vi.doMock()` of a
/// first-party module.
struct MockVisitor<'s> {
    file: &'s Path,
    source: &'s str,
    violations: Vec<Violation>,
}

impl MockVisitor<'_> {
    fn report(&mut self, span: Span, spec: &str) {
        self.violations.push(Violation {
            file: self.file.to_path_buf(),
            line: line_of(self.source, span.start),
            rule: "no-first-party-mock",
            message: format!(
                "integration test mocks first-party module `{spec}` — an integration test \
                 runs first-party code for real; only third-party packages and Node built-ins \
                 may be mocked"
            ),
        });
    }
}

impl<'a> Visit<'a> for MockVisitor<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Some(spec) = vi_mock_target(call) {
            if classify(&spec) == Origin::FirstParty {
                self.report(call.span, &spec);
            }
        }
        walk::walk_call_expression(self, call);
    }
}

/// If `call` is `vi.mock("spec", …)` or `vi.doMock("spec", …)` with a string
/// literal first argument, return that specifier; otherwise `None`.
///
/// A non-literal target (`vi.mock(name)`) can't be classified deterministically,
/// so it is skipped rather than guessed at.
fn vi_mock_target(call: &CallExpression) -> Option<String> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    let is_vi = matches!(&member.object, Expression::Identifier(id) if id.name == "vi");
    if !is_vi {
        return None;
    }
    let method = member.property.name.as_str();
    if method != "mock" && method != "doMock" {
        return None;
    }
    match call.arguments.first() {
        Some(Argument::StringLiteral(lit)) => Some(lit.value.to_string()),
        _ => None,
    }
}

/// The 1-based line containing byte `offset` in `source`.
fn line_of(source: &str, offset: u32) -> usize {
    let offset = (offset as usize).min(source.len());
    source.as_bytes()[..offset]
        .iter()
        .filter(|&&byte| byte == b'\n')
        .count()
        + 1
}

/// Recursively collect every TypeScript test file under `dir` into `out`.
fn collect_ts_test_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = entry
            .with_context(|| format!("reading an entry under `{}`", dir.display()))?
            .path();
        if path.is_dir() {
            collect_ts_test_files(&path, out)?;
        } else if is_ts_test_file(&path) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` for a TypeScript test file: `*.test.{ts,tsx,mts,cts}`.
fn is_ts_test_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name.ends_with(".test.ts")
        || name.ends_with(".test.tsx")
        || name.ends_with(".test.mts")
        || name.ends_with(".test.cts")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parse `source` as `name` and return its integration violations.
    fn violations(name: &str, source: &str) -> Vec<Violation> {
        integration_violations_in(Path::new(name), source).expect("source should parse")
    }

    /// Parse `source` as `name` and return its unit-isolation violations.
    fn unit_violations(name: &str, source: &str) -> Vec<Violation> {
        unit_violations_in(Path::new(name), source).expect("source should parse")
    }

    #[test]
    fn unit_flags_unmocked_first_party_and_external() {
        let found = unit_violations(
            "widget.test.ts",
            "import { makeWidget } from './widget';\n\
             import { format } from './formatter';\n\
             import { chunk } from 'lodash';\n",
        );
        // The unit under test (`./widget`) is not a collaborator; the other two are
        // imported but not mocked.
        assert_eq!(found.len(), 2, "got: {found:?}");
        assert!(found.iter().all(|v| v.rule == "unmocked-collaborator"));
        assert!(found.iter().any(|v| v.message.contains("./formatter")));
        assert!(found.iter().any(|v| v.message.contains("lodash")));
    }

    #[test]
    fn unit_mocked_collaborator_is_clean() {
        let found = unit_violations(
            "widget.test.ts",
            "import { format } from './formatter';\nvi.mock('./formatter');\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn unit_under_test_and_runner_are_not_flagged() {
        let found = unit_violations(
            "widget.test.ts",
            "import { vi } from 'vitest';\n\
             import { makeWidget } from './widget.js';\n",
        );
        // `vitest` is the runner; `./widget.js` is the unit under test (extension ignored).
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn unit_type_only_import_is_not_flagged() {
        let found = unit_violations(
            "widget.test.ts",
            "import type { Opts } from './opts';\nimport { x } from './x';\nvi.mock('./x');\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn unit_under_test_specifier_strips_test_suffix() {
        assert_eq!(
            unit_under_test_specifier(Path::new("pkg/widget.test.ts")),
            "./widget"
        );
        assert_eq!(
            unit_under_test_specifier(Path::new("button.test.tsx")),
            "./button"
        );
    }

    #[test]
    fn strip_module_ext_drops_known_extensions_only() {
        assert_eq!(strip_module_ext("./widget.js"), "./widget");
        assert_eq!(strip_module_ext("./widget.mts"), "./widget");
        assert_eq!(strip_module_ext("./widget"), "./widget");
        assert_eq!(strip_module_ext("lodash"), "lodash");
    }

    #[test]
    fn recognizes_the_test_runner() {
        assert!(is_test_runner("vitest"));
        assert!(is_test_runner("vitest/config"));
        assert!(is_test_runner("@vitest/spy"));
        assert!(!is_test_runner("./vitest-helpers"));
        assert!(!is_test_runner("lodash"));
    }

    #[test]
    fn unit_flags_untyped_factory_mock() {
        let found = unit_violations(
            "widget.test.ts",
            "import { x } from './x';\nvi.mock('./x', () => ({ x: vi.fn() }));\n",
        );
        // Mocked, so not an `unmocked-collaborator`; but the factory has no
        // `vi.importActual<…>` anchor.
        assert_eq!(found.len(), 1, "got: {found:?}");
        assert_eq!(found[0].rule, "untyped-mock");
        assert!(found[0].message.contains("./x"));
    }

    #[test]
    fn unit_typed_factory_mock_is_clean() {
        let found = unit_violations(
            "widget.test.ts",
            "import { x } from './x';\n\
             vi.mock('./x', async () => {\n\
             \x20 const actual = await vi.importActual<typeof import('./x')>('./x');\n\
             \x20 return { ...actual, x: vi.fn() };\n\
             });\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn unit_options_object_mock_is_not_a_factory() {
        // Vitest's options form `vi.mock(spec, { spy: true })` is not a factory —
        // it spies on the real module (can't drift), like a bare auto-mock — so it
        // must not be flagged `untyped-mock`.
        let found = unit_violations(
            "widget.test.ts",
            "import { x } from './x';\nvi.mock('./x', { spy: true });\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn unit_untyped_import_actual_is_still_untyped() {
        // `vi.importActual` without a type argument returns `unknown` — not a type anchor.
        let found = unit_violations(
            "widget.test.ts",
            "import { x } from './x';\n\
             vi.mock('./x', async () => {\n\
             \x20 const actual = await vi.importActual('./x');\n\
             \x20 return { ...(actual as object), x: vi.fn() };\n\
             });\n",
        );
        assert_eq!(found.len(), 1, "got: {found:?}");
        assert_eq!(found[0].rule, "untyped-mock");
    }

    #[test]
    fn classify_relative_is_first_party() {
        assert_eq!(classify("./service"), Origin::FirstParty);
        assert_eq!(classify("../pkg/util"), Origin::FirstParty);
        assert_eq!(classify("/abs/path"), Origin::FirstParty);
    }

    #[test]
    fn classify_node_builtins() {
        assert_eq!(classify("fs"), Origin::Builtin);
        assert_eq!(classify("node:fs"), Origin::Builtin);
        assert_eq!(classify("fs/promises"), Origin::Builtin);
        assert_eq!(classify("node:test"), Origin::Builtin);
        assert_eq!(classify("child_process"), Origin::Builtin);
        assert_eq!(classify("node:some-future-builtin"), Origin::Builtin);
    }

    #[test]
    fn classify_third_party() {
        assert_eq!(classify("lodash"), Origin::ThirdParty);
        assert_eq!(classify("@scope/pkg"), Origin::ThirdParty);
        assert_eq!(classify("stripe/lib/client"), Origin::ThirdParty);
        // A bare `test` is too ambiguous to assume the built-in; only `node:test`
        // is treated as a built-in.
        assert_eq!(classify("test"), Origin::ThirdParty);
    }

    #[test]
    fn recognizes_ts_test_files() {
        assert!(is_ts_test_file(Path::new("widget.test.ts")));
        assert!(is_ts_test_file(Path::new("pkg/button.test.tsx")));
        assert!(is_ts_test_file(Path::new("service.test.mts")));
        assert!(is_ts_test_file(Path::new("legacy.test.cts")));
        assert!(!is_ts_test_file(Path::new("widget.ts")));
        assert!(!is_ts_test_file(Path::new("types.d.ts")));
        assert!(!is_ts_test_file(Path::new("README.md")));
    }

    #[test]
    fn line_of_counts_newlines() {
        let src = "a\nb\nc\n";
        assert_eq!(line_of(src, 0), 1);
        assert_eq!(line_of(src, 2), 2);
        assert_eq!(line_of(src, 4), 3);
    }

    #[test]
    fn flags_mock_of_relative_module() {
        let found = violations("a.test.ts", "vi.mock('./service');\n");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].rule, "no-first-party-mock");
        assert_eq!(found[0].line, 1);
    }

    #[test]
    fn flags_mock_with_factory_and_parent_path() {
        let found = violations(
            "a.test.ts",
            "import { x } from './x';\nvi.mock('../src/ledger', () => ({ record: vi.fn() }));\n",
        );
        assert_eq!(found.len(), 1);
        assert!(found[0].message.contains("../src/ledger"));
    }

    #[test]
    fn flags_domock_of_relative_module() {
        let found = violations("a.test.mts", "vi.doMock('./mailer');\n");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn allows_mock_of_third_party_and_builtins() {
        let found = violations(
            "a.test.ts",
            "vi.mock('stripe');\nvi.mock('node:fs');\nvi.mock('fs/promises');\nvi.mock('@scope/pkg');\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn ignores_non_vi_and_non_mock_calls() {
        // `describe(...)` (plain call), `vi.fn()` (vi, not mock), and a method
        // call whose receiver isn't `vi` must all be left alone.
        let found = violations(
            "a.test.ts",
            "describe('s', () => {});\nvi.fn();\nexpect(1).toBe(1);\nother.mock('./x');\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn ignores_dynamic_mock_target() {
        // A non-literal specifier can't be classified deterministically.
        let found = violations("a.test.ts", "const m = './x';\nvi.mock(m);\n");
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn finds_mocks_nested_in_blocks() {
        // `vi.mock` is normally hoisted to the top level, but a nested call is
        // still reached by the walk.
        let found = violations(
            "a.test.ts",
            "describe('s', () => {\n  vi.mock('./inner');\n});\n",
        );
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].line, 2);
    }

    #[test]
    fn parse_error_is_reported() {
        let err = integration_violations_in(Path::new("bad.test.ts"), "const x = ;\n").unwrap_err();
        assert!(err.to_string().contains("parsing"), "got: {err}");
    }

    #[test]
    fn unsupported_extension_is_reported() {
        let err = integration_violations_in(Path::new("weird.test.bogus"), "vi.mock('./x');\n")
            .unwrap_err();
        assert!(err.to_string().contains("unsupported"), "got: {err}");
    }
}
