//! A TypeScript module split into its top-level import bindings and the emitted code of
//! everything else.

use std::collections::BTreeMap;
use std::path::Path;

use oxc::allocator::Allocator;
use oxc::ast::ast::{ImportDeclaration, ImportDeclarationSpecifier, Statement};
use oxc::parser::Parser;
use oxc::span::SourceType;

use crate::relocation::{walk_from, Binding, Split};
use crate::ts::{emit_without_comments, strip_module_ext};

const MODULE_EXTENSIONS: [&str; 8] = ["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"];

/// `source` at the repo-relative `path`, split. `None` when it fails to parse.
pub(crate) fn split(source: &str, path: &Path) -> Option<Split<String>> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).ok()?;
    let ret = Parser::new(&allocator, source, source_type).parse();
    if ret.panicked || !ret.diagnostics.is_empty() {
        return None;
    }
    let mut bindings = BTreeMap::new();
    let mut rest = source.to_string();
    for stmt in ret.program.body.iter().rev() {
        if let Statement::ImportDeclaration(decl) = stmt {
            bind(decl, path, &mut bindings);
            rest.replace_range(decl.span.start as usize..decl.span.end as usize, "");
        }
    }
    let rest = emit_without_comments(&rest, path)?;
    Some(Split { bindings, rest })
}

/// Binds each local name `decl` introduces. A side-effect import binds its specifier under a
/// key no identifier can take, so adding, dropping, or retargeting one changes the name set.
fn bind(decl: &ImportDeclaration, path: &Path, bindings: &mut BTreeMap<String, Binding>) {
    let source = decl.source.value.as_str();
    let paths = module_paths(source, path);
    let Some(specifiers) = &decl.specifiers else {
        let origin = source.to_string();
        bindings.insert(format!("import '{source}'"), Binding { origin, paths });
        return;
    };
    for specifier in specifiers {
        let (local, imported) = match specifier {
            ImportDeclarationSpecifier::ImportSpecifier(s) => (s.local.name, s.imported.name()),
            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                (s.local.name, "default".into())
            }
            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => (s.local.name, "*".into()),
        };
        let origin = format!("{source}:{imported}");
        let paths = paths.clone();
        bindings.insert(local.to_string(), Binding { origin, paths });
    }
}

/// The repo-relative files a relative `specifier` may name; empty for a package specifier.
fn module_paths(specifier: &str, path: &Path) -> Vec<String> {
    if !(specifier.starts_with("./") || specifier.starts_with("../")) {
        return Vec::new();
    }
    let Some(stem) = walk_from(path, 0, strip_module_ext(specifier).split('/')) else {
        return Vec::new();
    };
    MODULE_EXTENSIONS
        .iter()
        .flat_map(|ext| [format!("{stem}.{ext}"), format!("{stem}/index.{ext}")])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bindings(source: &str) -> Vec<(String, String)> {
        split(source, Path::new("src/reaper.ts"))
            .expect("the source splits")
            .bindings
            .into_iter()
            .map(|(name, b)| (name, b.origin))
            .collect()
    }

    fn pair(name: &str, origin: &str) -> (String, String) {
        (name.to_string(), origin.to_string())
    }

    #[test]
    fn every_specifier_kind_binds_its_local_name() {
        assert_eq!(
            bindings(
                "import def, { a, b as c } from './m.js';\nimport * as ns from 'pkg';\n\
                 import './side.js';\n"
            ),
            vec![
                pair("a", "./m.js:a"),
                pair("c", "./m.js:b"),
                pair("def", "./m.js:default"),
                pair("import './side.js'", "./side.js"),
                pair("ns", "pkg:*"),
            ]
        );
    }

    #[test]
    fn a_relative_specifier_maps_to_each_module_file() {
        let paths = module_paths("../old/ended.js", Path::new("src/sub/reaper.ts"));
        assert_eq!(paths.len(), 16);
        assert_eq!(paths[0], "src/old/ended.ts");
        assert_eq!(paths[1], "src/old/ended/index.ts");
        assert!(paths.contains(&"src/old/ended.mjs".to_string()));
        assert_eq!(module_paths("./old", Path::new("reaper.ts"))[0], "old.ts");
    }

    #[test]
    fn a_package_or_escaping_specifier_maps_to_no_file() {
        assert!(module_paths("pkg", Path::new("src/reaper.ts")).is_empty());
        assert!(module_paths("/abs/m.js", Path::new("src/reaper.ts")).is_empty());
        assert!(module_paths("../../m.js", Path::new("src/reaper.ts")).is_empty());
    }

    #[test]
    fn the_rest_drops_imports_and_keeps_code() {
        let path = Path::new("src/reaper.ts");
        let two = split(
            "import { a } from './a.js';\nimport { b } from './b.js';\nexport const z = a + b;\n",
            path,
        )
        .unwrap();
        let one = split(
            "import { a, b } from './c.js';\nexport const z = a + b;\n",
            path,
        )
        .unwrap();
        let changed = split(
            "import { a, b } from './c.js';\nexport const z = a - b;\n",
            path,
        )
        .unwrap();
        assert_eq!(two.rest, one.rest);
        assert_ne!(one.rest, changed.rest);
    }

    #[test]
    fn unparseable_source_does_not_split() {
        assert!(split("export const = ;\n", Path::new("src/reaper.ts")).is_none());
        assert!(split("x", Path::new("src/reaper.unknown")).is_none());
    }
}
