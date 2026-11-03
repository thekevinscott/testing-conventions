//! Whether an edit only relocates a module's imports, the co-change check's
//! language-neutral half. `python_imports` and `ts_imports` split each side.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Where one bound name comes from.
#[derive(Debug, PartialEq)]
pub(crate) struct Binding {
    /// The import as written: the module and the name taken from it.
    pub origin: String,
    /// The repo-relative paths the module may live at; empty when it maps to no file.
    pub paths: Vec<String>,
}

/// A module split into its top-level import bindings and the code left without them.
pub(crate) struct Split<R> {
    pub bindings: BTreeMap<String, Binding>,
    pub rest: R,
}

/// `true` when `head` differs from `base` only by imports that moved away from a module in
/// `deleted`: the same code outside the imports, and the same bound names.
pub(crate) fn is_relocation<R: PartialEq>(
    base: &Split<R>,
    head: &Split<R>,
    deleted: &BTreeSet<String>,
) -> bool {
    base.rest == head.rest
        && base.bindings.keys().eq(head.bindings.keys())
        && base
            .bindings
            .iter()
            .all(|(name, old)| old.origin == head.bindings[name].origin || moved_away(old, deleted))
}

/// `true` when one of the paths `old`'s module may live at is in `deleted`.
fn moved_away(old: &Binding, deleted: &BTreeSet<String>) -> bool {
    old.paths.iter().any(|path| {
        deleted
            .iter()
            .any(|gone| gone == path || gone.ends_with(&format!("/{path}")))
    })
}

/// The `/`-joined path reached from `file`'s directory by climbing `ups` levels, then following
/// `segments` (`.` stays, `..` climbs). `None` when the walk climbs above the root.
pub(crate) fn walk_from<'a>(
    file: &Path,
    ups: usize,
    segments: impl IntoIterator<Item = &'a str>,
) -> Option<String> {
    let mut parts: Vec<String> = file
        .parent()
        .into_iter()
        .flat_map(|dir| dir.iter())
        .map(|part| part.to_string_lossy().into_owned())
        .collect();
    for _ in 0..ups {
        parts.pop()?;
    }
    for segment in segments {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop()?;
            }
            name => parts.push(name.to_string()),
        }
    }
    Some(parts.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(origin: &str, paths: &[&str]) -> Binding {
        Binding {
            origin: origin.to_string(),
            paths: paths.iter().map(|p| p.to_string()).collect(),
        }
    }

    fn split(rest: &str, bindings: Vec<(&str, Binding)>) -> Split<String> {
        Split {
            bindings: bindings
                .into_iter()
                .map(|(name, b)| (name.to_string(), b))
                .collect(),
            rest: rest.to_string(),
        }
    }

    fn deleted(paths: &[&str]) -> BTreeSet<String> {
        paths.iter().map(|p| p.to_string()).collect()
    }

    #[test]
    fn a_binding_moved_off_a_deleted_module_is_a_relocation() {
        let base = split(
            "code",
            vec![("ended", binding("old:ended", &["pkg/old.py"]))],
        );
        let head = split(
            "code",
            vec![("ended", binding("new:ended", &["pkg/new.py"]))],
        );
        assert!(is_relocation(&base, &head, &deleted(&["pkg/old.py"])));
        assert!(is_relocation(&base, &head, &deleted(&["src/pkg/old.py"])));
    }

    #[test]
    fn a_binding_moved_off_a_live_module_is_not_a_relocation() {
        let base = split(
            "code",
            vec![("ended", binding("old:ended", &["pkg/old.py"]))],
        );
        let head = split(
            "code",
            vec![("ended", binding("new:ended", &["pkg/new.py"]))],
        );
        assert!(!is_relocation(&base, &head, &deleted(&[])));
        assert!(!is_relocation(&base, &head, &deleted(&["xpkg/old.py"])));
        assert!(!is_relocation(&base, &head, &deleted(&["pkg/new.py"])));
    }

    #[test]
    fn an_unmapped_module_is_never_moved_away_from() {
        let base = split("code", vec![("ended", binding("old:ended", &[]))]);
        let head = split("code", vec![("ended", binding("new:ended", &[]))]);
        assert!(!is_relocation(&base, &head, &deleted(&["pkg/old.py"])));
    }

    #[test]
    fn an_unchanged_binding_needs_no_deletion() {
        let base = split(
            "code",
            vec![("ended", binding("old:ended", &["pkg/old.py"]))],
        );
        let head = split(
            "code",
            vec![("ended", binding("old:ended", &["pkg/old.py"]))],
        );
        assert!(is_relocation(&base, &head, &deleted(&[])));
    }

    #[test]
    fn a_changed_name_set_or_rest_is_not_a_relocation() {
        let gone = deleted(&["pkg/old.py"]);
        let base = split(
            "code",
            vec![("ended", binding("old:ended", &["pkg/old.py"]))],
        );
        let other_name = split("code", vec![("done", binding("new:done", &["pkg/new.py"]))]);
        let other_rest = split(
            "more",
            vec![("ended", binding("new:ended", &["pkg/new.py"]))],
        );
        let extra = split(
            "code",
            vec![
                ("ended", binding("new:ended", &["pkg/new.py"])),
                ("settle", binding("new:settle", &["pkg/new.py"])),
            ],
        );
        assert!(!is_relocation(&base, &other_name, &gone));
        assert!(!is_relocation(&base, &other_rest, &gone));
        assert!(!is_relocation(&base, &extra, &gone));
        assert!(!is_relocation(&extra, &base, &gone));
    }

    #[test]
    fn walk_from_climbs_and_descends_from_the_files_directory() {
        let file = Path::new("pkg/sub/reaper.py");
        assert_eq!(
            walk_from(file, 0, ["old", "ended"]).unwrap(),
            "pkg/sub/old/ended"
        );
        assert_eq!(walk_from(file, 1, ["old"]).unwrap(), "pkg/old");
        assert_eq!(walk_from(file, 0, [".", "..", "old"]).unwrap(), "pkg/old");
        assert_eq!(walk_from(file, 0, ["", "old"]).unwrap(), "pkg/sub/old");
        assert_eq!(walk_from(file, 2, []).unwrap(), "");
    }

    #[test]
    fn walk_from_refuses_to_climb_above_the_root() {
        let file = Path::new("pkg/reaper.py");
        assert!(walk_from(file, 2, ["old"]).is_none());
        assert!(walk_from(file, 0, ["..", "..", "old"]).is_none());
    }
}
