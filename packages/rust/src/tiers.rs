//! The standard suite-tier layout, derived from the package root.

use std::path::{Path, PathBuf};

/// The manifests that mark a directory as a package root, one per language the checks support.
/// Every check that asks "where is the package?" asks about one of these files.
pub const MANIFESTS: [&str; 3] = ["package.json", "pyproject.toml", "Cargo.toml"];

/// `true` when `dir` is a package root — it holds one of [`MANIFESTS`]. The language-agnostic
/// form of [`package_root`], for a caller that has a candidate directory rather than a file.
pub fn is_package_root(dir: &Path) -> bool {
    MANIFESTS
        .iter()
        .any(|manifest| dir.join(manifest).is_file())
}

/// The nearest directory at or above `scan_root` holding `manifest`, or `None`.
/// The walk stops at a `.git` boundary so it cannot escape the repository.
pub fn package_root(scan_root: &Path, manifest: &str) -> Option<PathBuf> {
    for dir in scan_root.ancestors() {
        if dir.join(manifest).is_file() {
            return Some(dir.to_path_buf());
        }
        if dir.join(".git").exists() {
            break;
        }
    }
    None
}

/// The `<package root>/tests/` directory `scan_root` belongs to, or `None`.
pub fn suite_tests_dir(scan_root: &Path, manifest: &str) -> Option<PathBuf> {
    package_root(scan_root, manifest).map(|root| root.join("tests"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{is_package_root, package_root, suite_tests_dir, MANIFESTS};

    struct TempTree(PathBuf);

    impl TempTree {
        fn new() -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::temp_dir().join(format!(
                "tc-tiers-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir_all(&dir).unwrap();
            TempTree(dir)
        }

        fn touch(&self, name: &str) {
            let path = self.0.join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "").unwrap();
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn finds_the_nearest_manifest_above_the_scan_root() {
        let tree = TempTree::new();
        tree.touch("pkg/pyproject.toml");
        tree.touch("pkg/src/widget.py");
        assert_eq!(
            package_root(&tree.0.join("pkg/src"), "pyproject.toml"),
            Some(tree.0.join("pkg")),
        );
    }

    #[test]
    fn the_scan_root_itself_can_be_the_package_root() {
        let tree = TempTree::new();
        tree.touch("pkg/package.json");
        assert_eq!(
            package_root(&tree.0.join("pkg"), "package.json"),
            Some(tree.0.join("pkg")),
        );
    }

    #[test]
    fn the_walk_stops_at_a_git_boundary() {
        let tree = TempTree::new();
        tree.touch("Cargo.toml");
        tree.touch("repo/.git/HEAD");
        tree.touch("repo/src/lib.rs");
        assert_eq!(package_root(&tree.0.join("repo/src"), "Cargo.toml"), None);
    }

    #[test]
    fn a_manifest_at_the_git_boundary_is_still_found() {
        let tree = TempTree::new();
        tree.touch("repo/.git/HEAD");
        tree.touch("repo/pyproject.toml");
        assert_eq!(
            package_root(&tree.0.join("repo"), "pyproject.toml"),
            Some(tree.0.join("repo")),
        );
    }

    #[test]
    fn suite_tests_dir_is_the_package_roots_tests() {
        let tree = TempTree::new();
        tree.touch(".git/HEAD");
        tree.touch("pkg/pyproject.toml");
        assert_eq!(
            suite_tests_dir(&tree.0.join("pkg"), "pyproject.toml"),
            Some(tree.0.join("pkg/tests")),
        );
        assert_eq!(suite_tests_dir(&tree.0, "pyproject.toml"), None);
    }

    #[test]
    fn a_directory_holding_any_manifest_is_a_package_root() {
        let tree = TempTree::new();
        for (i, manifest) in MANIFESTS.iter().enumerate() {
            let dir = format!("pkg{i}");
            assert!(!is_package_root(&tree.0.join(&dir)));
            tree.touch(&format!("{dir}/{manifest}"));
            assert!(is_package_root(&tree.0.join(&dir)));
        }
    }

    #[test]
    fn a_directory_holding_no_manifest_is_not_a_package_root() {
        let tree = TempTree::new();
        tree.touch("docs/guide.md");
        assert!(!is_package_root(&tree.0.join("docs")));
    }
}
