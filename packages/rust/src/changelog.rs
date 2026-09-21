//! Changelog rule: a pull request that changes a package's public surface adds a fragment
//! recording it. The layout is discovered from where the fragment directories sit, so a
//! consumer declares nothing.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Where a repository keeps its fragments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Layout {
    /// One fragment directory per package; the payload is the directories holding the packages.
    PerPackage(Vec<String>),
    /// One fragment directory for the whole repository.
    Pooled,
}

/// The two fragment kinds, in the order the check reports them missing.
pub const KINDS: [&str; 2] = ["changelog", "migrations"];

/// One thing a pull request owes, ready to render as an annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// The offending file, where the finding has one.
    pub file: Option<String>,
    pub message: String,
}

/// Directories the fragment walk never descends into.
const SKIPPED_DIRS: [&str; 2] = ["node_modules", "target"];

/// The layout `root` keeps its fragments in, or `None` when it keeps none.
pub fn discover_layout(root: &Path) -> Option<Layout> {
    let dirs = fragment_dirs(root);
    if dirs.is_empty() {
        return None;
    }
    let mut containers: Vec<String> = dirs
        .iter()
        .filter(|segs| segs.len() == 3)
        .map(|segs| segs[0].clone())
        .collect();
    containers.sort();
    containers.dedup();
    if containers.is_empty() {
        return Some(Layout::Pooled);
    }
    Some(Layout::PerPackage(containers))
}

/// `true` when `root` keeps migration fragments alongside its changelog fragments.
pub fn migrations_enforced(root: &Path) -> bool {
    fragment_dirs(root)
        .iter()
        .any(|segs| segs.last().is_some_and(|last| last == "migrations.d"))
}

/// `true` when `name` is `YYYY-MM-DD-<slug>.md` — the UTC merge date, then lowercase letters,
/// digits and hyphens.
pub fn fragment_name_ok(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".md") else {
        return false;
    };
    let bytes = stem.as_bytes();
    if bytes.len() < 12 {
        return false;
    }
    let digit = |i: usize| bytes[i].is_ascii_digit();
    let dated = (0..4).all(digit)
        && bytes[4] == b'-'
        && (5..7).all(digit)
        && bytes[7] == b'-'
        && (8..10).all(digit)
        && bytes[10] == b'-';
    dated
        && bytes[11..]
            .iter()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'-')
}

/// `true` when any line of `bodies` opens with `skip-changelog:`.
pub fn has_skip_line(bodies: &str) -> bool {
    const SKIP: &[u8] = b"skip-changelog:";
    bodies.lines().any(|line| {
        let bytes = line.as_bytes();
        bytes.len() >= SKIP.len() && bytes[..SKIP.len()].eq_ignore_ascii_case(SKIP)
    })
}

/// `true` when `path` sits under `pkg` and is not public surface.
pub fn is_exempt(path: &str, pkg: &str) -> bool {
    path.strip_prefix(pkg)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(exempt_at_any_boundary)
}

/// The package directories `changed` touches, unique and sorted.
pub fn changed_packages(changed: &[String]) -> Vec<String> {
    let mut out: Vec<String> = changed
        .iter()
        .filter_map(|path| {
            let segs: Vec<&str> = path.split('/').collect();
            (segs.len() > 2).then(|| format!("{}/{}", segs[0], segs[1]))
        })
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Everything the pull request owes: fragments whose names break the convention, then the
/// fragments each changed scope is still missing.
pub fn findings(
    layout: &Layout,
    migrations: bool,
    changed: &[String],
    added: &[String],
) -> Vec<Finding> {
    let mut out: Vec<Finding> = malformed(layout, changed)
        .into_iter()
        .map(|path| Finding {
            file: Some(path),
            message: "fragment filenames are YYYY-MM-DD-<slug>.md — the UTC merge date, then \
                      lowercase letters, digits and hyphens. See docs/reference/checks/changelog."
                .to_string(),
        })
        .collect();

    match layout {
        Layout::PerPackage(containers) => {
            for pkg in changed_packages(changed) {
                let in_a_container = containers.iter().any(|c| pkg.split('/').next() == Some(c));
                if in_a_container && code_touched(changed, &pkg) {
                    for kind in missing_kinds(layout, added, Some(&pkg), migrations) {
                        out.push(owed(&format!("{pkg} "), &format!("{pkg}/{kind}.d"), kind));
                    }
                }
            }
        }
        Layout::Pooled => {
            if changed.iter().any(|path| !exempt_at_any_boundary(path)) {
                for kind in missing_kinds(layout, added, None, migrations) {
                    out.push(owed("", &format!("{kind}.d"), kind));
                }
            }
        }
    }
    out
}

/// The bodies of every commit in `<base>..HEAD`, concatenated.
pub fn commit_bodies(repo: &Path, base: &str) -> Result<String> {
    git(repo, &["log", "--format=%B", &format!("{base}..HEAD")])
}

/// Every path `<base>...HEAD` changed.
pub fn changed_files(repo: &Path, base: &str) -> Result<Vec<String>> {
    let out = git(repo, &["diff", "--name-only", &format!("{base}...HEAD")])?;
    Ok(lines(&out))
}

/// The paths `<base>...HEAD` added. A fragment satisfies the check only when the pull request
/// adds it, so the diff is filtered to additions.
pub fn added_files(repo: &Path, base: &str) -> Result<Vec<String>> {
    let range = format!("{base}...HEAD");
    let out = git(repo, &["diff", "--name-only", "--diff-filter=A", &range])?;
    Ok(lines(&out))
}

fn owed(scope: &str, dir: &str, kind: &str) -> Finding {
    Finding {
        file: None,
        message: format!(
            "{scope}changed public surface without adding a {kind} fragment. Add \
             {dir}/YYYY-MM-DD-<slug>.md, or put a `skip-changelog: <reason>` line on any commit \
             for a genuinely internal refactor. See docs/reference/checks/changelog."
        ),
    }
}

/// A fragment path, split into the scope that owns it, its kind, and its filename.
struct Fragment {
    /// The owning package, or `None` under the pooled layout.
    pkg: Option<String>,
    kind: &'static str,
    name: String,
}

fn fragment(path: &str, layout: &Layout) -> Option<Fragment> {
    let segs: Vec<&str> = path.split('/').collect();
    let i = segs.iter().position(|seg| kind_of(seg).is_some())?;
    let kind = kind_of(segs[i])?;
    if i + 2 != segs.len() {
        return None;
    }
    let name = segs[i + 1].to_string();
    match layout {
        Layout::PerPackage(containers) if i == 2 && containers.iter().any(|c| c == segs[0]) => {
            Some(Fragment {
                pkg: Some(format!("{}/{}", segs[0], segs[1])),
                kind,
                name,
            })
        }
        Layout::PerPackage(_) => None,
        Layout::Pooled => Some(Fragment {
            pkg: None,
            kind,
            name,
        }),
    }
}

fn kind_of(segment: &str) -> Option<&'static str> {
    let stem = segment.strip_suffix(".d")?;
    KINDS.into_iter().find(|kind| *kind == stem)
}

/// Touched fragment paths whose filenames break the convention. Each fragment directory carries
/// a `README.md` describing that convention, which is not an entry.
fn malformed(layout: &Layout, changed: &[String]) -> Vec<String> {
    changed
        .iter()
        .filter(|path| {
            fragment(path, layout)
                .is_some_and(|frag| frag.name != "README.md" && !fragment_name_ok(&frag.name))
        })
        .cloned()
        .collect()
}

fn missing_kinds(
    layout: &Layout,
    added: &[String],
    pkg: Option<&str>,
    migrations: bool,
) -> Vec<&'static str> {
    let present: Vec<&'static str> = added
        .iter()
        .filter_map(|path| fragment(path, layout))
        .filter(|frag| fragment_name_ok(&frag.name) && frag.pkg.as_deref() == pkg)
        .map(|frag| frag.kind)
        .collect();
    KINDS
        .into_iter()
        .filter(|kind| migrations || *kind != "migrations")
        .filter(|kind| !present.contains(kind))
        .collect()
}

fn code_touched(changed: &[String], pkg: &str) -> bool {
    let prefix = format!("{pkg}/");
    changed
        .iter()
        .any(|path| path.starts_with(&prefix) && !is_exempt(path, pkg))
}

fn exempt_at_any_boundary(rel: &str) -> bool {
    std::iter::once(rel)
        .chain(rel.match_indices('/').map(|(i, _)| &rel[i + 1..]))
        .any(exempt_shape)
}

fn exempt_shape(rel: &str) -> bool {
    matches!(rel, "CHANGELOG.md" | "MIGRATIONS.md")
        || KINDS
            .iter()
            .any(|kind| rel.starts_with(&format!("{kind}.d/")))
        || rel.starts_with("e2e-attestations/")
        || (rel.contains('/')
            && matches!(rel.split('/').next(), Some("tests" | "test" | "__tests__")))
        || rel.ends_with("_test.py")
        || is_test_or_spec(rel)
}

fn is_test_or_spec(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    ["ts", "tsx", "js", "mjs", "cjs", "py", "rs"]
        .iter()
        .any(|ext| {
            name.ends_with(&format!(".test.{ext}")) || name.ends_with(&format!(".spec.{ext}"))
        })
}

/// Every fragment directory under `root`, as its root-relative segments. The convention puts a
/// fragment directory at most two levels down, which bounds the walk.
fn fragment_dirs(root: &Path) -> Vec<Vec<String>> {
    let mut out = Vec::new();
    scan(root, &[], &mut out);
    out
}

fn scan(dir: &Path, prefix: &[String], out: &mut Vec<Vec<String>>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') || SKIPPED_DIRS.contains(&name.as_str()) {
            continue;
        }
        let mut segs = prefix.to_vec();
        segs.push(name.clone());
        if kind_of(&name).is_some() {
            out.push(segs);
        } else if segs.len() < 3 {
            scan(&entry.path(), &segs, out);
        }
    }
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .with_context(|| format!("running `git {}` in `{}`", args.join(" "), repo.display()))?;
    if !out.status.success() {
        bail!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn lines(out: &str) -> Vec<String> {
    out.lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}
