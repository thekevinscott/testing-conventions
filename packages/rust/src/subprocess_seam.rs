//! Which of a Rust file's functions are subprocess seams: branch-free bodies that run a
//! `std::process::Command`, directly or through another function in the same file. A mutant that
//! replaces such a body whole has no unit-tier contract — calling the function spawns.

use std::collections::BTreeSet;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};

/// The lines `source`'s subprocess seams cover. Unparseable source yields nothing, so a mutant
/// there stays a survivor rather than vanishing silently.
pub(crate) fn subprocess_seam_lines(source: &str) -> BTreeSet<u32> {
    let Ok(ast) = syn::parse_file(source) else {
        return BTreeSet::new();
    };
    let mut shapes = Shapes::default();
    shapes.visit_file(&ast);
    seam_lines(&shapes.0)
}

/// One function as the seam rule reads it: where it sits, whether its body holds a decision,
/// whether it runs a `Command` itself, and which same-file functions it calls.
struct Shape {
    name: String,
    first: u32,
    last: u32,
    branch_free: bool,
    runs_a_command: bool,
    calls: BTreeSet<String>,
}

/// Every function in the file, in source order.
#[derive(Default)]
struct Shapes(Vec<Shape>);

impl<'ast> Visit<'ast> for Shapes {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.0.push(shape(&node.sig, &node.block, node.span()));
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.0.push(shape(&node.sig, &node.block, node.span()));
        visit::visit_impl_item_fn(self, node);
    }
}

fn shape(sig: &syn::Signature, block: &syn::Block, span: proc_macro2::Span) -> Shape {
    let mut body = Body::default();
    body.visit_signature(sig);
    body.visit_block(block);
    Shape {
        name: sig.ident.to_string(),
        first: span.start().line as u32,
        last: span.end().line as u32,
        branch_free: !body.branches,
        runs_a_command: body.executes && body.names_a_command,
        calls: body.calls,
    }
}

/// What a walk of one body found. Control flow sets `branches` and is not descended into: a
/// conditional spawn is not a seam, so what hides inside the branch does not matter.
#[derive(Default)]
struct Body {
    branches: bool,
    executes: bool,
    names_a_command: bool,
    calls: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for Body {
    fn visit_expr_if(&mut self, _: &'ast syn::ExprIf) {
        self.branches = true;
    }

    fn visit_expr_match(&mut self, _: &'ast syn::ExprMatch) {
        self.branches = true;
    }

    fn visit_expr_loop(&mut self, _: &'ast syn::ExprLoop) {
        self.branches = true;
    }

    fn visit_expr_while(&mut self, _: &'ast syn::ExprWhile) {
        self.branches = true;
    }

    fn visit_expr_for_loop(&mut self, _: &'ast syn::ExprForLoop) {
        self.branches = true;
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {
        self.branches = true;
    }

    fn visit_expr_try_block(&mut self, _: &'ast syn::ExprTryBlock) {
        self.branches = true;
    }

    fn visit_local(&mut self, node: &'ast syn::Local) {
        if node
            .init
            .as_ref()
            .is_some_and(|init| init.diverge.is_some())
        {
            self.branches = true;
        }
        visit::visit_local(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if matches!(
            node.method.to_string().as_str(),
            "output" | "status" | "spawn"
        ) {
            self.executes = true;
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(path) = &*node.func {
            if let Some(name) = same_file_call(&path.path) {
                self.calls.insert(name);
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_ident(&mut self, node: &'ast syn::Ident) {
        if node == "Command" {
            self.names_a_command = true;
        }
    }

    fn visit_item_fn(&mut self, _: &'ast syn::ItemFn) {}
}

/// The called function's name when the call could name a function in this file — a bare `run(…)`.
/// A qualified path (`Command::new`, `String::from_utf8_lossy`) names something else.
fn same_file_call(path: &syn::Path) -> Option<String> {
    match path.segments.len() {
        1 => Some(path.segments[0].ident.to_string()),
        _ => None,
    }
}

/// The lines of every branch-free function that reaches a spawn. Reaching is transitive: a
/// function whose body only calls a spawner spawns too, and the walk repeats until it settles.
fn seam_lines(shapes: &[Shape]) -> BTreeSet<u32> {
    let spawning = spawning(shapes);
    shapes
        .iter()
        .filter(|shape| shape.branch_free && spawning.contains(&shape.name))
        .flat_map(|shape| shape.first..=shape.last)
        .collect()
}

/// The names of the functions that run a `Command`, directly or through one another.
fn spawning(shapes: &[Shape]) -> BTreeSet<String> {
    let mut reached: BTreeSet<String> = shapes
        .iter()
        .filter(|shape| shape.runs_a_command)
        .map(|shape| shape.name.clone())
        .collect();
    loop {
        let added: Vec<String> = shapes
            .iter()
            .filter(|shape| !reached.contains(&shape.name))
            .filter(|shape| shape.calls.iter().any(|call| reached.contains(call)))
            .map(|shape| shape.name.clone())
            .collect();
        if added.is_empty() {
            return reached;
        }
        reached.extend(added);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_body_that_only_spawns_is_a_seam() {
        let source = "\
fn probe() -> bool {
    Command::new(\"true\").output().is_ok()
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::from([1, 2, 3]));
    }

    #[test]
    fn a_seam_covers_its_doc_comment_and_attributes() {
        let source = "\
/// Probe.
#[inline]
fn probe() -> bool {
    Command::new(\"true\").output().is_ok()
}
";
        assert_eq!(
            subprocess_seam_lines(source),
            BTreeSet::from([1, 2, 3, 4, 5])
        );
    }

    #[test]
    fn a_body_that_delegates_to_a_seam_is_a_seam() {
        let source = "\
fn probe() -> bool {
    Command::new(\"true\").output().is_ok()
}

fn probe_again() -> bool {
    probe()
}
";
        assert_eq!(
            subprocess_seam_lines(source),
            BTreeSet::from([1, 2, 3, 5, 6, 7])
        );
    }

    #[test]
    fn delegation_carries_through_a_chain() {
        let source = "\
fn probe() -> bool {
    Command::new(\"true\").output().is_ok()
}

fn once() -> bool {
    probe()
}

fn twice() -> bool {
    once()
}
";
        assert!(subprocess_seam_lines(source).contains(&10));
    }

    #[test]
    fn a_branch_before_the_spawn_is_not_a_seam() {
        let source = "\
fn probe(verbose: bool) -> bool {
    let mut command = Command::new(\"true\");
    if verbose {
        command.arg(\"--verbose\");
    }
    command.output().is_ok()
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::new());
    }

    #[test]
    fn a_match_in_the_body_is_a_branch() {
        let source = "\
fn probe(flag: Option<&str>) -> bool {
    let mut command = Command::new(\"true\");
    match flag {
        Some(flag) => command.arg(flag),
        None => &mut command,
    }
    .output()
    .is_ok()
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::new());
    }

    #[test]
    fn a_let_else_is_a_branch() {
        let source = "\
fn probe(flag: Option<&str>) -> bool {
    let Some(flag) = flag else {
        return false;
    };
    Command::new(\"true\").arg(flag).output().is_ok()
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::new());
    }

    #[test]
    fn a_body_that_runs_no_command_is_not_a_seam() {
        let source = "\
fn verdict(ok: bool) -> &'static str {
    describe(ok)
}

fn describe(ok: bool) -> &'static str {
    ok.then_some(\"up\").unwrap_or(\"down\")
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::new());
    }

    #[test]
    fn spawning_a_thread_is_not_spawning_a_process() {
        let source = "\
fn probe() -> bool {
    std::thread::spawn(|| ()).join().is_ok()
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::new());
    }

    #[test]
    fn a_command_the_caller_owns_still_makes_a_seam() {
        let source = "\
fn execute(command: &mut Command) -> std::io::Result<Output> {
    command.output()
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::from([1, 2, 3]));
    }

    #[test]
    fn a_method_is_a_subject_too() {
        let source = "\
impl Engine {
    fn probe(&self) -> bool {
        Command::new(\"true\").output().is_ok()
    }
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::from([2, 3, 4]));
    }

    #[test]
    fn a_nested_function_keeps_its_spawn_to_itself() {
        let source = "\
fn verdict() -> bool {
    fn probe() -> bool {
        Command::new(\"true\").output().is_ok()
    }
    true
}
";
        assert_eq!(subprocess_seam_lines(source), BTreeSet::from([2, 3, 4]));
    }

    #[test]
    fn unparseable_source_yields_no_seams() {
        assert_eq!(subprocess_seam_lines("fn probe( {"), BTreeSet::new());
    }
}
