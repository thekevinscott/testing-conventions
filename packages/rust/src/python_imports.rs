//! A Python module split into its top-level import bindings and the tokens of everything else.

use std::collections::BTreeMap;
use std::path::Path;

use rustpython_parser::lexer::lex;
use rustpython_parser::text_size::TextRange;
use rustpython_parser::{ast, Mode, Parse, Tok};

use crate::relocation::{walk_from, Binding, Split};

/// `source` at the repo-relative `path`, split. `None` when it fails to parse or holds an
/// `import *`, whose bound names can't be read.
pub(crate) fn split(source: &str, path: &Path) -> Option<Split<Vec<Tok>>> {
    let suite = ast::Suite::parse(source, "<source>").ok()?;
    let mut bindings = BTreeMap::new();
    let mut ranges = Vec::new();
    for stmt in &suite {
        match stmt {
            ast::Stmt::Import(import) => {
                bind_imports(&import.names, &mut bindings);
                ranges.push(import.range);
            }
            ast::Stmt::ImportFrom(from) => {
                bind_from(from, path, &mut bindings)?;
                ranges.push(from.range);
            }
            _ => {}
        }
    }
    let rest = tokens_outside(source, &ranges)?;
    Some(Split { bindings, rest })
}

/// Binds each `import a.b [as c]`: `c`, or the dotted name itself, to module `a.b`.
fn bind_imports(names: &[ast::Alias], bindings: &mut BTreeMap<String, Binding>) {
    for alias in names {
        let module = alias.name.as_str();
        let paths = module_paths(&module.replace('.', "/"));
        let name = alias.asname.as_ref().unwrap_or(&alias.name).to_string();
        bindings.insert(
            name,
            Binding {
                origin: module.to_string(),
                paths,
            },
        );
    }
}

/// Binds each name of `from <module> import x [as y]`. `None` for `import *`.
fn bind_from(
    from: &ast::StmtImportFrom,
    path: &Path,
    bindings: &mut BTreeMap<String, Binding>,
) -> Option<()> {
    let level = from.level.map_or(0, |level| level.to_u32() as usize);
    let module = from.module.as_ref().map_or("", |m| m.as_str());
    let dir = module_dir(level, module, path);
    for alias in &from.names {
        let name = alias.name.as_str();
        if name == "*" {
            return None;
        }
        let paths = dir
            .as_ref()
            .map_or_else(Vec::new, |dir| from_paths(dir, name));
        let origin = format!("{}{module}:{name}", ".".repeat(level));
        let bound = alias.asname.as_ref().map_or(name, |a| a.as_str());
        bindings.insert(bound.to_string(), Binding { origin, paths });
    }
    Some(())
}

/// The `/`-joined directory path of `module`, relative imports resolved against `path`.
fn module_dir(level: usize, module: &str, path: &Path) -> Option<String> {
    let segments: Vec<&str> = module.split('.').filter(|s| !s.is_empty()).collect();
    if level == 0 {
        return Some(segments.join("/"));
    }
    walk_from(path, level - 1, &segments)
}

/// Where `from <dir> import <name>` may read from: the module itself, or a submodule `name`.
fn from_paths(dir: &str, name: &str) -> Vec<String> {
    let mut paths = module_paths(dir);
    paths.extend(module_paths(&format!("{dir}/{name}")));
    paths
}

/// A module at `stem` lives at `stem.py` or `stem/__init__.py`.
fn module_paths(stem: &str) -> Vec<String> {
    let stem = stem.trim_start_matches('/');
    if stem.is_empty() {
        return vec!["__init__.py".to_string()];
    }
    vec![format!("{stem}.py"), format!("{stem}/__init__.py")]
}

/// The tokens of `source` outside `ranges`, each range's closing newline dropped with it.
fn tokens_outside(source: &str, ranges: &[TextRange]) -> Option<Vec<Tok>> {
    let mut tokens = Vec::new();
    let mut closing = false;
    for item in lex(source, Mode::Module) {
        let (tok, range) = item.ok()?;
        if ranges.iter().any(|outer| outer.contains_range(range)) {
            closing = true;
            continue;
        }
        if !std::mem::take(&mut closing) || tok != Tok::Newline {
            tokens.push(tok);
        }
    }
    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings(source: &str, path: &str) -> Vec<(String, String, Vec<String>)> {
        split(source, Path::new(path))
            .expect("the source splits")
            .bindings
            .into_iter()
            .map(|(name, b)| (name, b.origin, b.paths))
            .collect()
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn an_absolute_from_import_maps_to_the_module_and_its_submodule() {
        assert_eq!(
            bindings("from pkg.old import ended\n", "pkg/reaper.py"),
            vec![(
                "ended".to_string(),
                "pkg.old:ended".to_string(),
                strings(&[
                    "pkg/old.py",
                    "pkg/old/__init__.py",
                    "pkg/old/ended.py",
                    "pkg/old/ended/__init__.py",
                ]),
            )]
        );
    }

    #[test]
    fn a_relative_from_import_resolves_against_the_file() {
        let found = bindings("from ..old import ended as done\n", "pkg/sub/reaper.py");
        assert_eq!(found[0].0, "done");
        assert_eq!(found[0].1, "..old:ended");
        assert_eq!(found[0].2[0], "pkg/old.py");
    }

    #[test]
    fn a_bare_relative_import_reads_the_package() {
        let found = bindings("from . import old\n", "pkg/reaper.py");
        assert_eq!(found[0].1, ".:old");
        assert_eq!(
            found[0].2,
            strings(&[
                "pkg.py",
                "pkg/__init__.py",
                "pkg/old.py",
                "pkg/old/__init__.py"
            ])
        );
        let top = bindings("from . import old\n", "reaper.py");
        assert_eq!(
            top[0].2,
            strings(&["__init__.py", "old.py", "old/__init__.py"])
        );
    }

    #[test]
    fn a_relative_import_above_the_root_maps_to_no_file() {
        assert!(bindings("from ... import old\n", "reaper.py")[0]
            .2
            .is_empty());
    }

    #[test]
    fn a_plain_import_binds_its_alias_or_its_dotted_name() {
        assert_eq!(
            bindings("import pkg.old\nimport pkg.new as fresh\n", "reaper.py"),
            vec![
                (
                    "fresh".to_string(),
                    "pkg.new".to_string(),
                    strings(&["pkg/new.py", "pkg/new/__init__.py"]),
                ),
                (
                    "pkg.old".to_string(),
                    "pkg.old".to_string(),
                    strings(&["pkg/old.py", "pkg/old/__init__.py"]),
                ),
            ]
        );
    }

    #[test]
    fn a_star_import_or_unparseable_source_does_not_split() {
        assert!(split("from pkg import *\n", Path::new("reaper.py")).is_none());
        assert!(split("def f() return 1\n", Path::new("reaper.py")).is_none());
    }

    #[test]
    fn the_rest_drops_top_level_imports_and_their_newlines() {
        let path = Path::new("reaper.py");
        let four = split("from a import x\nfrom b import y\n\nz = x + y\n", path).unwrap();
        let one = split("from c import x, y\nz = x + y\n", path).unwrap();
        let bare = split("z = x + y\n", path).unwrap();
        assert!(four.rest == one.rest && one.rest == bare.rest);
    }

    #[test]
    fn the_rest_keeps_nested_imports_and_code() {
        let path = Path::new("reaper.py");
        let nested = split("def f():\n    import a\n    return a\n", path).unwrap();
        let moved = split("def f():\n    import b as a\n    return a\n", path).unwrap();
        assert!(nested.bindings.is_empty());
        assert!(nested.rest != moved.rest);
        let changed = split("from a import x\nz = 2\n", path).unwrap();
        let kept = split("from a import x\nz = 1\n", path).unwrap();
        assert!(changed.rest != kept.rest);
    }
}
