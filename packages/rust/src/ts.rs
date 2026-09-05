//! TypeScript isolation analysis, parsed with `oxc` — the counterpart to the Python
//! [`crate::lint`] module. Each `*.test.{ts,tsx,mts,cts}` file is parsed and walked, and its
//! specifiers are [`classify`]-ed first-party / Node-builtin / third-party.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use oxc::allocator::Allocator;
use oxc::ast::ast::{
    Argument, CallExpression, Expression, ImportDeclaration, ImportOrExportKind, Statement,
};
use oxc::ast_visit::{walk, Visit};
use oxc::parser::Parser;
use oxc::span::{SourceType, Span};
use oxc_codegen::{Codegen, CodegenOptions, CommentOptions};

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

/// Classify a module specifier, resolution-free: a relative or absolute path is first-party,
/// a `node:` prefix or a built-in head segment is a built-in, and every other bare specifier
/// is a third-party package.
pub fn classify(specifier: &str) -> Origin {
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return Origin::FirstParty;
    }
    if specifier.starts_with("node:") || is_node_builtin(specifier) {
        return Origin::Builtin;
    }
    Origin::ThirdParty
}

/// `true` when `specifier`'s head segment is a Node built-in, so `fs/promises` matches on `fs`.
fn is_node_builtin(specifier: &str) -> bool {
    let head = specifier.split('/').next().unwrap_or(specifier);
    NODE_BUILTINS.contains(&head)
}

/// The Node.js built-in module names. An explicit `node:` prefix is handled in [`classify`],
/// so a future built-in stays recognized when written `node:<name>`.
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

/// Every integration-isolation violation in the `*.test.{ts,tsx,mts,cts}` files under
/// `root`, sorted by `(file, line)`. A file that cannot be read or parsed is an error.
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

const UNKNOWN_TIER_MSG: &str = "test file sits under `tests/` outside the standard suite tiers; \
     a suite lives in `tests/integration/` or `tests/e2e/`";

/// Every integration-isolation violation in `package_root`'s suite tiers, sorted by
/// `(file, line)`. `tests/integration/` and `tests/e2e/` both run first-party code for real;
/// a test file under `tests/` outside them is `unknown-tier` rather than silently unscanned.
pub fn find_suite_violations(package_root: &Path) -> Result<Vec<Violation>> {
    let tests = package_root.join("tests");
    let mut violations = Vec::new();
    let tiers = ["integration", "e2e"].map(|tier| tests.join(tier));
    for tier in &tiers {
        if tier.is_dir() {
            violations.extend(find_integration_violations(tier)?);
        }
    }
    if tests.is_dir() {
        let mut strays = Vec::new();
        collect_ts_test_files(&tests, &mut strays)?;
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

/// Every unit-isolation violation under `root` — a runtime import that isn't `vi.mock()`-ed
/// — sorted by `(file, line)`. A file that cannot be read or parsed is an error.
pub fn find_unit_violations(root: impl AsRef<Path>) -> Result<Vec<Violation>> {
    let root = root.as_ref();
    let mut files = Vec::new();
    collect_ts_test_files(root, &mut files)?;
    // The suite tiers run first-party code for real, so their files are never unit subjects.
    if let Some(tests) = crate::tiers::suite_tests_dir(root, "package.json") {
        files.retain(|file| !file.starts_with(&tests));
    }
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

/// One unit test file's `unmocked-collaborator` violations: every runtime import that isn't
/// the unit under test, the test runner, or `vi.mock()`-ed.
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
    // Vitest resolves `./formatter` and `./formatter.js` to one module, so an extension
    // mismatch between a mock and its import must not read as an unmocked collaborator.
    let mocked_modules: BTreeSet<&str> = collector
        .mocked
        .iter()
        .map(|m| strip_module_ext(m))
        .collect();
    let mut violations = Vec::new();
    for (spec, line) in &collector.imports {
        if is_unit_under_test(spec, &unit)
            || is_test_runner(spec)
            || mocked_modules.contains(strip_module_ext(spec))
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

/// Collects a unit test's imports, `vi.mock()` targets, and untyped factories in one pass.
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

/// `true` when `spec` resolves to the unit under test, ignoring the module extension.
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

/// `true` for the Vitest runner itself (`vitest`, `vitest/*`, `@vitest/*`), never a mock target.
fn is_test_runner(spec: &str) -> bool {
    spec == "vitest" || spec.starts_with("vitest/") || spec.starts_with("@vitest/")
}

/// `true` when a `vi.mock` second argument is a factory *function*. The other 2nd-arg form
/// is an options object (`vi.mock(spec, { spy: true })`), which spies on the real module and
/// so can't drift; only a function factory returns a hand-built double that can.
fn is_factory(arg: &Argument) -> bool {
    matches!(
        arg.as_expression(),
        Some(Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_))
    )
}

/// `true` when a `vi.mock` factory anchors to the real module's type — its body contains a
/// `vi.importActual<…>()` call carrying a type argument.
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

/// One test file's `no-first-party-mock` violations. A parse failure is an error — a
/// malformed test file is never a silent pass.
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

/// Walks one test file, flagging every `vi.mock()` / `vi.doMock()` of a first-party module.
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

/// The specifier of a `vi.mock("spec", …)` / `vi.doMock("spec", …)` call, or `None`. A
/// non-literal target (`vi.mock(name)`) can't be classified deterministically, so it is
/// skipped rather than guessed at.
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

/// `true` when `source` (a module at `path`) declares something and compiles to zero runtime
/// JavaScript, so it has no behavior to unit-test. An empty module and one that fails to parse
/// are both `false`: the presence rule keeps a module it couldn't read as a subject.
pub fn is_type_only_module(source: &str, path: &Path) -> bool {
    let allocator = Allocator::default();
    let Ok(source_type) = SourceType::from_path(path) else {
        return false;
    };
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked || !ret.diagnostics.is_empty() {
        return false;
    }
    let body = &ret.program.body;
    !body.is_empty() && body.iter().all(is_type_only_statement)
}

/// `true` when a top-level statement contributes no runtime code — see [`is_type_only_module`].
fn is_type_only_statement(statement: &Statement) -> bool {
    match statement {
        Statement::TSTypeAliasDeclaration(_) | Statement::TSInterfaceDeclaration(_) => true,
        Statement::ImportDeclaration(decl) => decl.import_kind.is_type(),
        Statement::ExportAllDeclaration(decl) => decl.export_kind.is_type(),
        // The parser marks an exported type alias or interface as a type export, so a
        // value-kind named export always carries runtime bindings.
        Statement::ExportNamedDeclaration(decl) => decl.export_kind.is_type(),
        _ => false,
    }
}

/// `true` when `base` and `head` — the module at `path` before and after an edit — compile to
/// the same JavaScript. A side that fails to parse is **not** equal: co-change then holds the
/// file to its colocated test rather than skip a module it couldn't read.
pub fn same_code(base: &str, head: &str, path: &Path) -> bool {
    match (
        emit_without_comments(base, path),
        emit_without_comments(head, path),
    ) {
        (Some(base), Some(head)) => base == head,
        _ => false,
    }
}

/// `source` re-emitted with every comment dropped, or `None` when it does not parse.
fn emit_without_comments(source: &str, path: &Path) -> Option<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).ok()?;
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked || !ret.diagnostics.is_empty() {
        return None;
    }
    Some(
        Codegen::new()
            .with_options(CodegenOptions {
                comments: CommentOptions::disabled(),
                ..CodegenOptions::default()
            })
            .build(&ret.program)
            .code,
    )
}

fn collect_ts_test_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = crate::walk::dir_entry(entry, dir)?.path();
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
        // `./widget` is the unit under test; the other two are imported but not mocked.
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

    /// Whether `source` (named `foo.ts`) is a type-only module.
    fn type_only(source: &str) -> bool {
        is_type_only_module(source, Path::new("foo.ts"))
    }

    #[test]
    fn type_only_recognizes_a_pure_type_module() {
        assert!(type_only(
            "export interface Shape { kind: string }\nexport type Id = string;\n"
        ));
        assert!(type_only(
            "import type { Shape } from './shape';\nexport type Wrapped = Shape;\n"
        ));
        assert!(type_only("export type { Id } from './shape';\n"));
        assert!(type_only(
            "type Local = number;\ninterface Bare { x: Local }\n"
        ));
        assert!(type_only("export type * from './shapes';\n"));
    }

    #[test]
    fn type_only_rejects_any_runtime_construct() {
        assert!(!type_only(
            "export type T = number;\nexport const version: T = 1;\n"
        ));
        assert!(!type_only(
            "export interface I {}\nexport function make(): I { return {}; }\n"
        ));
        assert!(!type_only(
            "import { x } from './x';\nexport type T = typeof x;\n"
        ));
        assert!(!type_only("export * from './widget';\n"));
        assert!(!type_only("export { thing } from './thing';\n"));
        assert!(!type_only("export enum Color { Red, Green }\n"));
        assert!(!type_only("export namespace N { export const x = 1; }\n"));
    }

    #[test]
    fn type_only_is_false_for_empty_or_unparsable() {
        // An empty/comment-only file is a non-subject on its own account, not via this path.
        assert!(!type_only(""));
        assert!(!type_only("// just a comment\n"));
        assert!(!type_only("export type T = ;;;\nconst {{{ = \n"));
    }

    /// Whether `base` and `head` (a module named `foo.ts`) compile to the same JavaScript.
    fn same(base: &str, head: &str) -> bool {
        same_code(base, head, Path::new("foo.ts"))
    }

    #[test]
    fn same_code_drops_comments_and_formatting() {
        assert!(same(
            "// widget factory\nexport const widget = () => 1;\n",
            "// widget builder\nexport const widget = () => 1;\n"
        ));
        assert!(same(
            "/* widget factory\n   used by the CLI */\nexport const widget = () => 1;\n",
            "export const widget = () => 1;\n"
        ));
        assert!(same(
            "/** A widget. */\nexport const widget = () => 1;\n",
            "export const widget = () => 1;\n"
        ));
        assert!(same(
            "export const widget = () => 1;\n",
            "\n\nexport const widget = () => 1;\n\n"
        ));
        assert!(same(
            "export function widget() { return 1; }\n",
            "export function widget() {\n        return 1;\n}\n"
        ));
    }

    #[test]
    fn same_code_keeps_everything_the_module_emits() {
        assert!(!same(
            "export const widget = () => 1;\n",
            "export const widget = () => 2;\n"
        ));
        assert!(!same(
            "export const widget = () => 'one';\n",
            "export const widget = () => 'two';\n"
        ));
        assert!(!same(
            "export const widget = () => `one`;\n",
            "export const widget = () => `two`;\n"
        ));
        assert!(!same(
            "export const widget = (n: number): number => n;\n",
            "export const widget = (n: string): string => n;\n"
        ));
        assert!(!same(
            "// widget factory\nexport const widget = () => 1;\n",
            "// widget builder\nexport const widget = () => 2;\n"
        ));
    }

    #[test]
    fn same_code_holds_apart_what_it_cannot_read() {
        assert!(!same(
            "export const widget = (() => 1;\n",
            "// still broken\nexport const widget = (() => 1;\n"
        ));
        assert!(!same(
            "export const widget = () => 1;\n",
            "export const widget = (() => 1;\n"
        ));
        assert!(!same(
            "export const widget = (() => 1;\n",
            "export const widget = () => 1;\n"
        ));
        let source = "export const widget = () => 1;\n";
        assert!(!same_code(source, source, Path::new("widget.txt")));
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
        // Mocked, so not an `unmocked-collaborator`; the factory has no type anchor.
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
        // A bare `test` is too ambiguous to assume the built-in; `node:test` is not.
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
        let found = violations(
            "a.test.ts",
            "describe('s', () => {});\nvi.fn();\nexpect(1).toBe(1);\nother.mock('./x');\n",
        );
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn ignores_dynamic_mock_target() {
        let found = violations("a.test.ts", "const m = './x';\nvi.mock(m);\n");
        assert!(found.is_empty(), "got: {found:?}");
    }

    #[test]
    fn finds_mocks_nested_in_blocks() {
        // `vi.mock` is normally hoisted, but a nested call is still reached by the walk.
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

    #[test]
    fn unit_parse_error_is_reported() {
        let err = unit_violations_in(Path::new("bad.test.ts"), "const x = ;\n").unwrap_err();
        assert!(err.to_string().contains("parsing"), "got: {err}");
    }

    #[test]
    fn unit_unsupported_extension_is_reported() {
        let err =
            unit_violations_in(Path::new("weird.test.bogus"), "vi.mock('./x');\n").unwrap_err();
        assert!(err.to_string().contains("unsupported"), "got: {err}");
    }

    #[test]
    fn type_only_is_false_for_an_unsupported_extension() {
        assert!(!is_type_only_module(
            "export type T = number;\n",
            Path::new("foo.txt")
        ));
    }

    #[test]
    fn type_only_rejects_a_plain_runtime_statement() {
        assert!(!type_only("const x = 1;\ntype T = number;\n"));
    }

    #[test]
    fn a_factory_calling_a_plain_helper_is_untyped() {
        let found = unit_violations(
            "widget.test.ts",
            "import { x } from './x';\nvi.mock('./x', () => makeDouble());\n",
        );
        assert_eq!(found.len(), 1, "got: {found:?}");
        assert_eq!(found[0].rule, "untyped-mock");
    }

    #[test]
    fn a_package_without_a_tests_dir_has_no_suite_violations() {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "tc-ts-suite-test-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let found = find_suite_violations(&dir).expect("an empty package scans clean");
        assert!(found.is_empty(), "got: {found:?}");
    }
}
