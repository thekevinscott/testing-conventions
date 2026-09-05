//! Packaging rule — the deterministic core: given the root of an unpacked built artifact and
//! the test-file globs that must not appear in it, [`scan`] returns every offending file.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// Every file under `root` — the root of an unpacked built artifact — whose name matches one
/// of `globs`, sorted. `globs` are file-name globs where `*` matches any run of characters;
/// each is matched against an entry's file name, not its full path.
pub fn scan(root: impl AsRef<Path>, globs: &[String]) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();
    let mut offenders = Vec::new();
    collect_offenders(root, root, globs, &mut offenders)?;
    offenders.sort();
    Ok(offenders)
}

/// Inspect a built artifact at `path` for files matching `globs`. `path` is a directory (an
/// already-unpacked artifact) or an archive this rule unpacks first — `.whl`, `.tgz`/`.tar.gz`,
/// `.crate`. Offenders come back as paths **relative to the artifact root**.
pub fn inspect(path: impl AsRef<Path>, globs: &[String]) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();
    if path.is_dir() {
        return Ok(relative_to(path, scan(path, globs)?));
    }
    let unpacked = if is_zip_artifact(path) {
        unzip_to_temp(path)?
    } else if is_tar_gz_artifact(path) {
        untar_gz_to_temp(path)?
    } else {
        bail!(
            "`{}` is not a directory or a recognized built artifact \
             (expected a directory, a `.whl`, a `.tgz`/`.tar.gz`, or a `.crate`)",
            path.display()
        )
    };
    Ok(relative_to(unpacked.path(), scan(unpacked.path(), globs)?))
}

/// `true` for an artifact this rule unpacks as a zip: a Python wheel (`.whl`) or a `.zip`.
fn is_zip_artifact(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("whl" | "zip")
    )
}

/// Re-express each offender as a path relative to `root`; an unexpected path is kept as-is.
fn relative_to(root: &Path, offenders: Vec<PathBuf>) -> Vec<PathBuf> {
    offenders
        .into_iter()
        .map(|p| p.strip_prefix(root).map(Path::to_path_buf).unwrap_or(p))
        .collect()
}

/// Unpack a zip artifact into a fresh scratch directory (removed on drop).
fn unzip_to_temp(archive: &Path) -> Result<TempDir> {
    let file = std::fs::File::open(archive)
        .with_context(|| format!("opening artifact `{}`", archive.display()))?;
    let mut zip = zip::ZipArchive::new(file)
        .with_context(|| format!("reading `{}` as a zip archive", archive.display()))?;
    let dir = TempDir::new()?;
    zip.extract(dir.path())
        .with_context(|| format!("unpacking `{}`", archive.display()))?;
    Ok(dir)
}

/// `true` for an artifact this rule unpacks as a gzipped tar: `.tgz`, `.tar.gz`, `.crate`.
fn is_tar_gz_artifact(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    name.ends_with(".tgz") || name.ends_with(".tar.gz") || name.ends_with(".crate")
}

/// Unpack a gzipped-tar artifact into a fresh scratch directory (removed on drop).
fn untar_gz_to_temp(archive: &Path) -> Result<TempDir> {
    let file = std::fs::File::open(archive)
        .with_context(|| format!("opening artifact `{}`", archive.display()))?;
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(file));
    let dir = TempDir::new()?;
    tar.unpack(dir.path())
        .with_context(|| format!("unpacking `{}`", archive.display()))?;
    Ok(dir)
}

/// A scratch directory removed on drop, unique per call so parallel checks never collide.
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Result<Self> {
        Self::new_in(&std::env::temp_dir())
    }

    fn new_in(base: &Path) -> Result<Self> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let path = base.join(format!(
            "testing-conventions-pkg-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&path)
            .with_context(|| format!("creating scratch directory `{}`", path.display()))?;
        Ok(TempDir(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Recursively collect every file under `dir` (within the artifact `root`) that
/// matches one of `patterns`.
fn collect_offenders(
    dir: &Path,
    root: &Path,
    patterns: &[String],
    out: &mut Vec<PathBuf>,
) -> Result<()> {
    let entries =
        std::fs::read_dir(dir).with_context(|| format!("reading directory `{}`", dir.display()))?;
    for entry in entries {
        let path = crate::walk::dir_entry(entry, dir)?.path();
        if path.is_dir() {
            collect_offenders(&path, root, patterns, out)?;
        } else if matches_any(&path, root, patterns) {
            out.push(path);
        }
    }
    Ok(())
}

/// `true` when `path` matches any of `patterns`. A pattern ending in `/` is a **directory**
/// pattern — it matches when `path` (relative to `root`) lives under a directory of that name;
/// every other pattern is a file-name glob (`*` wildcards) matched against the entry's name.
fn matches_any(path: &Path, root: &Path, patterns: &[String]) -> bool {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    patterns
        .iter()
        .any(|pattern| match pattern.strip_suffix('/') {
            Some(dir) => path_under_dir(path, root, dir),
            None => matches_glob(pattern, name),
        })
}

/// `true` when `path` (relative to `root`) has an **ancestor** directory named `dir`.
fn path_under_dir(path: &Path, root: &Path, dir: &str) -> bool {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative
        .parent()
        .is_some_and(|parents| parents.components().any(|c| c.as_os_str() == dir))
}

/// Match `name` against a file-name `glob` where `*` matches any run of characters (including
/// none) and every other character is literal. Matching is over Unicode scalar values.
fn matches_glob(glob: &str, name: &str) -> bool {
    let glob: Vec<char> = glob.chars().collect();
    let name: Vec<char> = name.chars().collect();
    // Linear wildcard match: on a mismatch, backtrack to the most recent `*` and extend
    // what it consumed by one character.
    let (mut g, mut n) = (0usize, 0usize);
    let mut star: Option<usize> = None;
    let mut consumed_by_star = 0usize;
    while n < name.len() {
        if g < glob.len() && glob[g] == name[n] {
            g += 1;
            n += 1;
        } else if g < glob.len() && glob[g] == '*' {
            star = Some(g);
            consumed_by_star = n;
            g += 1;
        } else if let Some(star) = star {
            g = star + 1;
            consumed_by_star += 1;
            n = consumed_by_star;
        } else {
            return false;
        }
    }
    // The pattern matches iff what's left is only trailing `*`s (each empty).
    while g < glob.len() && glob[g] == '*' {
        g += 1;
    }
    g == glob.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempTree(PathBuf);

    impl TempTree {
        fn new(files: &[&str]) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tc-packaging-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            for rel in files {
                let path = root.join(rel);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, "x").unwrap();
            }
            TempTree(root)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn star_matches_any_run_including_empty() {
        assert!(matches_glob("*", ""));
        assert!(matches_glob("*", "anything.py"));
        assert!(matches_glob("*.py", ".py"));
    }

    #[test]
    fn the_python_test_glob_matches_only_test_files() {
        assert!(matches_glob("*_test.py", "widget_test.py"));
        assert!(!matches_glob("*_test.py", "widget.py"));
        assert!(!matches_glob("*_test.py", "widget_test.pyc"));
    }

    #[test]
    fn the_typescript_test_glob_matches_across_extensions() {
        assert!(matches_glob("*.test.*", "button.test.ts"));
        assert!(matches_glob("*.test.*", "button.test.mts"));
        assert!(matches_glob("*.test.*", "button.test.tsx"));
        assert!(!matches_glob("*.test.*", "button.ts"));
    }

    #[test]
    fn a_literal_glob_must_match_exactly() {
        assert!(matches_glob("conftest.py", "conftest.py"));
        assert!(!matches_glob("conftest.py", "conftest.pyi"));
        assert!(!matches_glob("conftest.py", "xconftest.py"));
    }

    #[test]
    fn scan_flags_a_test_file_anywhere_in_the_tree() {
        let tree = TempTree::new(&["pkg/widget.py", "pkg/sub/helper_test.py"]);
        let offenders = scan(tree.path(), &["*_test.py".to_string()]).unwrap();
        assert_eq!(offenders, vec![tree.path().join("pkg/sub/helper_test.py")]);
    }

    #[test]
    fn a_directory_pattern_flags_files_under_that_dir() {
        let tree = TempTree::new(&["tests/integration.rs", "src/lib.rs", "src/tests/nested.rs"]);
        let offenders = scan(tree.path(), &["tests/".to_string()]).unwrap();
        assert_eq!(
            offenders,
            vec![
                tree.path().join("src/tests/nested.rs"),
                tree.path().join("tests/integration.rs"),
            ],
        );
    }

    #[test]
    fn recognizes_a_dot_crate_as_a_gzipped_tar() {
        assert!(is_tar_gz_artifact(Path::new("widget-0.1.0.crate")));
        assert!(is_tar_gz_artifact(Path::new("pkg.tgz")));
        assert!(is_tar_gz_artifact(Path::new("pkg.tar.gz")));
        assert!(!is_tar_gz_artifact(Path::new("pkg.whl")));
    }

    #[test]
    fn scan_is_clean_when_nothing_matches() {
        let tree = TempTree::new(&["pkg/widget.py", "pkg/helper.py"]);
        let offenders = scan(tree.path(), &["*_test.py".to_string()]).unwrap();
        assert!(offenders.is_empty());
    }

    #[test]
    fn scan_matches_any_of_several_globs_and_returns_sorted() {
        let tree = TempTree::new(&["a.test.ts", "b_test.py", "keep.ts"]);
        let globs = vec!["*_test.py".to_string(), "*.test.*".to_string()];
        let offenders = scan(tree.path(), &globs).unwrap();
        assert_eq!(
            offenders,
            vec![tree.path().join("a.test.ts"), tree.path().join("b_test.py")],
        );
    }

    #[test]
    fn scan_errors_when_the_root_cannot_be_read() {
        let missing = std::env::temp_dir().join("tc-packaging-does-not-exist-9f8e7d");
        assert!(scan(&missing, &["*_test.py".to_string()]).is_err());
    }

    #[test]
    fn inspect_scans_a_directory_artifact_with_relative_paths() {
        let tree = TempTree::new(&["pkg/widget.py", "pkg/widget_test.py"]);
        let offenders = inspect(tree.path(), &["*_test.py".to_string()]).unwrap();
        assert_eq!(offenders, vec![PathBuf::from("pkg/widget_test.py")]);
    }

    #[test]
    fn inspect_rejects_an_unrecognized_artifact() {
        let tree = TempTree::new(&["not-an-archive.txt"]);
        let artifact = tree.path().join("not-an-archive.txt");
        let err = inspect(artifact.as_path(), &["*_test.py".to_string()]).unwrap_err();
        assert!(
            err.to_string().contains("not a directory or a recognized"),
            "got: {err}"
        );
    }

    fn write_zip(path: &Path, entries: &[&str]) {
        use std::io::Write;
        let file = std::fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        for entry in entries {
            writer.start_file(*entry, options).unwrap();
            writer.write_all(b"x").unwrap();
        }
        writer.finish().unwrap();
    }

    fn write_tar_gz(path: &Path, entries: &[&str]) {
        let file = std::fs::File::create(path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for entry in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(1);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, *entry, &b"x"[..]).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap();
    }

    #[test]
    fn inspect_unpacks_a_wheel_and_reports_relative_offenders() {
        let tree = TempTree::new(&[]);
        std::fs::create_dir_all(tree.path()).unwrap();
        let wheel = tree.path().join("pkg.whl");
        write_zip(&wheel, &["pkg/widget.py", "pkg/widget_test.py"]);
        let offenders = inspect(wheel.as_path(), &["*_test.py".to_string()]).unwrap();
        assert_eq!(offenders, vec![PathBuf::from("pkg/widget_test.py")]);
    }

    #[test]
    fn inspect_unpacks_a_tarball_and_reports_relative_offenders() {
        let tree = TempTree::new(&[]);
        std::fs::create_dir_all(tree.path()).unwrap();
        let tarball = tree.path().join("pkg.tgz");
        write_tar_gz(&tarball, &["pkg/widget.py", "pkg/widget_test.py"]);
        let offenders = inspect(tarball.as_path(), &["*_test.py".to_string()]).unwrap();
        assert_eq!(offenders, vec![PathBuf::from("pkg/widget_test.py")]);
    }

    #[test]
    fn a_missing_wheel_reports_the_open_failure() {
        let Err(err) = unzip_to_temp(Path::new("/nonexistent-tc-packaging/pkg.whl")) else {
            panic!("expected an open failure");
        };
        assert!(err.to_string().contains("opening artifact"), "got: {err}");
    }

    #[test]
    fn a_wheel_that_is_not_a_zip_reports_the_read_failure() {
        let tree = TempTree::new(&["pkg.whl"]);
        let Err(err) = unzip_to_temp(&tree.path().join("pkg.whl")) else {
            panic!("expected a zip read failure");
        };
        assert!(err.to_string().contains("as a zip archive"), "got: {err}");
    }

    #[test]
    fn a_wheel_that_cannot_be_unpacked_reports_the_failure() {
        let tree = TempTree::new(&[]);
        std::fs::create_dir_all(tree.path()).unwrap();
        let wheel = tree.path().join("pkg.whl");
        write_zip(&wheel, &["a", "a/b"]);
        let Err(err) = unzip_to_temp(&wheel) else {
            panic!("expected an unpack failure");
        };
        assert!(err.to_string().contains("unpacking"), "got: {err}");
    }

    #[test]
    fn a_missing_tarball_reports_the_open_failure() {
        let Err(err) = untar_gz_to_temp(Path::new("/nonexistent-tc-packaging/pkg.tgz")) else {
            panic!("expected an open failure");
        };
        assert!(err.to_string().contains("opening artifact"), "got: {err}");
    }

    #[test]
    fn a_tarball_that_cannot_be_unpacked_reports_the_failure() {
        let tree = TempTree::new(&["pkg.tgz"]);
        let Err(err) = untar_gz_to_temp(&tree.path().join("pkg.tgz")) else {
            panic!("expected an unpack failure");
        };
        assert!(err.to_string().contains("unpacking"), "got: {err}");
    }

    #[test]
    fn an_uncreatable_scratch_directory_is_an_error() {
        let tree = TempTree::new(&["occupied"]);
        let Err(err) = TempDir::new_in(&tree.path().join("occupied")) else {
            panic!("expected a scratch-directory failure");
        };
        assert!(
            err.to_string().contains("creating scratch directory"),
            "got: {err}"
        );
    }
}
