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
    /// One fragment directory for the whole repository; the payload is the package roots whose
    /// surface those fragments pay for, empty when the tree has no discoverable package root.
    Pooled(Vec<String>),
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

/// Everything the changelog verdict reads from the repository.
///
/// Gathered in one go rather than on demand. The lazy shape — skip the git calls when no
/// fragment directory exists — cannot be split into a testable decision and an untestable
/// seam, because the laziness *is* the branch. Every repository behaves identically either
/// way: a repository with no fragment directories still reports "skipped", because the
/// layout, not the diff, decides that. Only a path that is not a repository at all changes,
/// and it now errors rather than reporting a clean skip.
struct Facts {
    layout: Option<Layout>,
    bodies: String,
    changed: Vec<String>,
    added: Vec<String>,
}

/// Read the repository facts the verdict decides on.
fn facts(root: &Path, base: &str) -> Result<Facts> {
    Ok(Facts {
        layout: discover_layout(root),
        bodies: commit_bodies(root, base)?,
        changed: changed_files(root, base)?,
        added: added_files(root, base)?,
    })
}

/// The exit code and the lines to print for gathered `facts`. The whole decision, with no
/// repository in reach: `0` for a tree keeping no fragments, `0` for a bypass, `0` when every
/// scope paid, and `1` with one annotation per scope that did not.
fn verdict(root: &Path, facts: &Facts) -> (i32, Vec<String>) {
    let Some(layout) = &facts.layout else {
        return (
            0,
            vec![format!(
                "No fragment directories under `{}`; changelog check skipped.",
                root.display()
            )],
        );
    };
    if has_skip_line(&facts.bodies) {
        return (
            0,
            vec!["A `skip-changelog:` line is present; changelog check bypassed.".to_string()],
        );
    }
    let breaking = has_breaking_line(&facts.bodies);
    let found = findings(layout, breaking, &facts.changed, &facts.added);
    if found.is_empty() {
        return (
            0,
            vec!["Every scope that changed public surface added its fragments.".to_string()],
        );
    }
    (1, found.iter().map(annotation).collect())
}

/// Report every scope in `<base>...HEAD` that changed public surface without adding the
/// fragments recording it. `0` when `root` keeps no fragment directories.
///
/// Branch-free on purpose: it reads the repository through [`facts`], so replacing its body
/// whole has no unit-tier contract. [`verdict`] holds the decision and is tested directly.
pub fn run(root: &Path, base: &str) -> Result<i32> {
    let (code, lines) = verdict(root, &facts(root, base)?);
    println!("{}", lines.join("\n"));
    Ok(code)
}

/// `finding` as the GitHub Actions annotation that reports it: file-scoped when the finding
/// names a file, workflow-scoped when it does not.
pub fn annotation(finding: &Finding) -> String {
    match &finding.file {
        Some(file) => format!("::error file={file}::{}", finding.message),
        None => format!("::error::{}", finding.message),
    }
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
        return Some(Layout::Pooled(package_roots(root)));
    }
    Some(Layout::PerPackage(containers))
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

/// `true` when any line of `bodies` opens with `marker`, whatever its case.
fn has_line_opening(bodies: &str, marker: &[u8]) -> bool {
    bodies.lines().any(|line| {
        let bytes = line.as_bytes();
        bytes.len() >= marker.len() && bytes[..marker.len()].eq_ignore_ascii_case(marker)
    })
}

/// `true` when any line of `bodies` opens with `skip-changelog:` — the bypass.
pub fn has_skip_line(bodies: &str) -> bool {
    has_line_opening(bodies, b"skip-changelog:")
}

/// `true` when any line of `bodies` opens with `breaking:` — the signal that the pull request
/// owes a migration fragment. Opt in: a breaking change whose commits carry no such line gets
/// no migrations enforcement, which is the trade for the check needing no configuration.
pub fn has_breaking_line(bodies: &str) -> bool {
    has_line_opening(bodies, b"breaking:")
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
///
/// `migrations` is whether the pull request marked a breaking change — see
/// [`has_breaking_line`]. A migration fragment is required when it did, and allowed, never
/// forbidden, when it did not.
pub fn findings(
    layout: &Layout,
    migrations: bool,
    changed: &[String],
    added: &[String],
) -> Vec<Finding> {
    let mut out = malformed_findings(layout, changed);
    out.extend(owed_findings(layout, migrations, changed, added));
    out
}

/// A finding per touched fragment whose filename breaks the convention.
fn malformed_findings(layout: &Layout, changed: &[String]) -> Vec<Finding> {
    malformed(layout, changed)
        .into_iter()
        .map(|path| Finding {
            file: Some(path),
            message: "fragment filenames are YYYY-MM-DD-<slug>.md — the UTC merge date, then \
                      lowercase letters, digits and hyphens. See docs/reference/checks/changelog."
                .to_string(),
        })
        .collect()
}

/// A finding per fragment a changed scope still owes. The per-package layout asks only a package
/// sitting in a declared container; the pooled layout scopes public surface to its discovered roots.
fn owed_findings(
    layout: &Layout,
    migrations: bool,
    changed: &[String],
    added: &[String],
) -> Vec<Finding> {
    let mut out = Vec::new();
    out.extend(match layout {
        Layout::PerPackage(containers) => {
            per_package_findings(layout, containers, migrations, changed, added)
        }
        Layout::Pooled(roots) => pooled_findings(layout, roots, migrations, changed, added),
    });
    out
}

/// What each changed package under one of `containers` still owes. The fragment directory sits
/// inside the package, so the directory names the package and the filename need not.
fn per_package_findings(
    layout: &Layout,
    containers: &[String],
    migrations: bool,
    changed: &[String],
    added: &[String],
) -> Vec<Finding> {
    let mut out = Vec::new();
    for pkg in changed_packages(changed) {
        let in_a_container = containers.iter().any(|c| pkg.split('/').next() == Some(c));
        if in_a_container && code_touched(changed, &pkg) {
            for kind in missing_kinds(layout, added, Some(&pkg), migrations) {
                out.push(owed(&format!("{pkg} "), &format!("{pkg}/{kind}.d"), kind));
            }
        }
    }
    out
}

/// What each changed package root still owes, when one pooled directory holds every package's
/// fragments. Public surface is scoped to `roots`, so a path under none of them owes nothing; a
/// tree with no discoverable root is one package at its own root, whose whole surface counts.
fn pooled_findings(
    layout: &Layout,
    roots: &[String],
    migrations: bool,
    changed: &[String],
    added: &[String],
) -> Vec<Finding> {
    let mut out = Vec::new();
    for root in pooled_scopes(roots) {
        if !pooled_surface_touched(changed, root) {
            continue;
        }
        let pkg = root.map(package_name);
        for kind in missing_kinds(layout, added, pkg, migrations) {
            out.push(owed_pooled(root, pkg, kind));
        }
    }
    out
}

/// The scopes a pooled tree answers for: each package root, or the tree itself when it has none.
fn pooled_scopes(roots: &[String]) -> Vec<Option<&str>> {
    if roots.is_empty() {
        return vec![None];
    }
    roots.iter().map(|root| Some(root.as_str())).collect()
}

/// `true` when `changed` touches the public surface of pooled scope `root`.
fn pooled_surface_touched(changed: &[String], root: Option<&str>) -> bool {
    match root {
        Some(root) => code_touched(changed, root),
        None => changed.iter().any(|path| !exempt_at_any_boundary(path)),
    }
}

/// The name a fragment calls package `root` by: its own directory name.
fn package_name(root: &str) -> &str {
    root.rsplit('/').next().unwrap_or(root)
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

/// What a pull request missing a fragment of `kind` did, and the way out of owing it. A lookup
/// with a default rather than a comparison, so the two kinds read side by side.
fn phrasing(kind: &str) -> (&'static str, &'static str) {
    match kind {
        "migrations" => (
            "marked a breaking change",
            "drop the `breaking:` line if the change is not breaking",
        ),
        _ => (
            "changed public surface",
            "put a `skip-changelog: <reason>` line on any commit for a genuinely internal \
             refactor",
        ),
    }
}

fn owed(scope: &str, dir: &str, kind: &str) -> Finding {
    named(scope, &format!("{dir}/YYYY-MM-DD-<slug>.md"), kind)
}

/// What pooled scope `root` owes. One directory holds every package's fragments, so the fragment
/// that pays names the package in its filename rather than by sitting inside it.
fn owed_pooled(root: Option<&str>, pkg: Option<&str>, kind: &str) -> Finding {
    let scope = root.map_or_else(String::new, |root| format!("{root} "));
    let slug = pkg.map_or_else(|| "<slug>".to_string(), |pkg| format!("{pkg}-<slug>"));
    named(&scope, &format!("{kind}.d/YYYY-MM-DD-{slug}.md"), kind)
}

/// The finding for a scope that owes a `kind` fragment, naming `fragment` as the file that pays.
fn named(scope: &str, fragment: &str, kind: &str) -> Finding {
    let (did, way_out) = phrasing(kind);
    Finding {
        file: None,
        message: format!(
            "{scope}{did} without adding a {kind} fragment. Add {fragment}, or {way_out}. See \
             docs/reference/checks/changelog."
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
    let pkg = fragment_scope(&segs, i, layout)?;
    Some(Fragment {
        pkg,
        kind,
        name: segs[i + 1].to_string(),
    })
}

/// The scope owning a fragment whose kind directory sits at `segs[i]`: the package under the
/// per-package layout, `Some(None)` under the pooled one. `None` where the layout places no
/// fragment there at all, so a stray path is not read as an entry.
fn fragment_scope(segs: &[&str], i: usize, layout: &Layout) -> Option<Option<String>> {
    match layout {
        Layout::PerPackage(containers) if i == 2 && containers.iter().any(|c| c == segs[0]) => {
            Some(Some(format!("{}/{}", segs[0], segs[1])))
        }
        Layout::PerPackage(_) => None,
        Layout::Pooled(_) => Some(None),
    }
}

/// `true` when `frag` pays for `scope`. The per-package layout reads the directory the fragment
/// sits in, and `scope` is the package's path; the pooled layout reads the filename, and `scope`
/// is the package's name. The filename rule cannot fire under the per-package layout, because
/// that arm never consults the name.
fn records(layout: &Layout, frag: &Fragment, scope: Option<&str>) -> bool {
    match layout {
        Layout::PerPackage(_) => frag.pkg.as_deref() == scope,
        Layout::Pooled(_) => names_package(&frag.name, scope),
    }
}

/// `true` when fragment `name` names package `pkg`: the slug after the date opens with `<pkg>-`.
/// A pooled tree with no package root has no name to carry, so any well-formed name records it.
///
/// `name` has passed [`fragment_name_ok`], so the date prefix is the first 11 bytes and the rest
/// is the slug.
fn names_package(name: &str, pkg: Option<&str>) -> bool {
    let Some(pkg) = pkg else {
        return true;
    };
    name.strip_suffix(".md")
        .and_then(|stem| stem.get(11..))
        .and_then(|slug| slug.strip_prefix(pkg))
        .is_some_and(|rest| rest.starts_with('-'))
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
    scope: Option<&str>,
    migrations: bool,
) -> Vec<&'static str> {
    let present: Vec<&'static str> = added
        .iter()
        .filter_map(|path| fragment(path, layout))
        .filter(|frag| fragment_name_ok(&frag.name) && records(layout, frag, scope))
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

/// Every package root under `root`, as a root-relative path, sorted. A package is a directory
/// inside a container directory — `packages/parser`, `crates/lexer` — holding one of the
/// manifests [`crate::tiers::is_package_root`] reads, which is the same question every other
/// check asks when it resolves a package root.
fn package_roots(root: &Path) -> Vec<String> {
    let mut out: Vec<String> = child_dirs(root)
        .iter()
        .flat_map(|container| packages_in(root, container))
        .collect();
    out.sort();
    out
}

/// The package roots directly under `root/container`, as root-relative paths.
fn packages_in(root: &Path, container: &str) -> Vec<String> {
    let dir = root.join(container);
    child_dirs(&dir)
        .into_iter()
        .filter(|name| crate::tiers::is_package_root(&dir.join(name)))
        .map(|name| format!("{container}/{name}"))
        .collect()
}

/// The names of `dir`'s visible subdirectories, skipping the build trees the walk never enters.
fn child_dirs(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.') && !SKIPPED_DIRS.contains(&name.as_str()))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn facts_with(layout: Option<Layout>, bodies: &str, changed: &[&str], added: &[&str]) -> Facts {
        Facts {
            layout,
            bodies: bodies.to_string(),
            changed: changed.iter().map(|s| s.to_string()).collect(),
            added: added.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn no_layout_skips_the_check_and_names_the_tree() {
        let facts = facts_with(None, "", &["packages/parser/src/a.ts"], &[]);
        let (code, lines) = verdict(Path::new("/repo"), &facts);
        assert_eq!(code, 0);
        assert_eq!(
            lines,
            vec!["No fragment directories under `/repo`; changelog check skipped."]
        );
    }

    #[test]
    fn a_skip_line_bypasses_a_tree_that_would_otherwise_owe_fragments() {
        let owing = &["packages/parser/src/a.ts"];
        let layout = Some(Layout::PerPackage(vec!["packages".to_string()]));
        // Same facts, with and without the trailer: the trailer is the only difference.
        assert_eq!(
            verdict(
                Path::new("/repo"),
                &facts_with(layout.clone(), "", owing, &[])
            )
            .0,
            1
        );
        let (code, lines) = verdict(
            Path::new("/repo"),
            &facts_with(
                layout,
                "refactor: rename\n\nskip-changelog: no public surface",
                owing,
                &[],
            ),
        );
        assert_eq!(code, 0);
        assert_eq!(
            lines,
            vec!["A `skip-changelog:` line is present; changelog check bypassed."]
        );
    }

    #[test]
    fn a_breaking_line_makes_a_migrations_fragment_required() {
        let layout = Some(Layout::PerPackage(vec!["packages".to_string()]));
        let changed = &["packages/parser/src/a.ts"];
        let paid = &["packages/parser/changelog.d/2026-10-06-change-a.md"];
        // Same facts, with and without the line: the line is the only difference.
        assert_eq!(
            verdict(
                Path::new("/repo"),
                &facts_with(layout.clone(), "", changed, paid)
            )
            .0,
            0
        );
        let (code, lines) = verdict(
            Path::new("/repo"),
            &facts_with(
                layout,
                "feat!: drop the flag\n\nbreaking: the flag is gone",
                changed,
                paid,
            ),
        );
        assert_eq!(code, 1);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("migrations fragment"));
        assert!(lines[0].contains("breaking change"));
    }

    #[test]
    fn a_migrations_fragment_no_breaking_line_asked_for_is_allowed() {
        // The requirement is an implication, not an equality: an unsolicited migration
        // fragment is allowed.
        let facts = facts_with(
            Some(Layout::PerPackage(vec!["packages".to_string()])),
            "",
            &["packages/parser/src/a.ts"],
            &[
                "packages/parser/changelog.d/2026-10-06-change-a.md",
                "packages/parser/migrations.d/2026-10-06-change-a.md",
            ],
        );
        assert_eq!(verdict(Path::new("/repo"), &facts).0, 0);
    }

    #[test]
    fn a_breaking_line_is_read_only_at_the_opening_of_a_line() {
        assert!(has_breaking_line("breaking: the flag is gone"));
        assert!(has_breaking_line(
            "feat!: drop it\n\nBREAKING: the flag is gone"
        ));
        assert!(!has_breaking_line(
            "fix: a comment saying this is not breaking: really"
        ));
        assert!(!has_breaking_line("breaking the flag"));
        assert!(!has_breaking_line(""));
    }

    #[test]
    fn phrasing_tells_a_breaking_change_apart_from_a_surface_change() {
        assert_eq!(
            phrasing("migrations"),
            (
                "marked a breaking change",
                "drop the `breaking:` line if the change is not breaking"
            )
        );
        let (did, way_out) = phrasing("changelog");
        assert_eq!(did, "changed public surface");
        assert!(way_out.contains("skip-changelog: <reason>"));
    }

    #[test]
    fn a_scope_that_added_its_fragment_passes() {
        let facts = facts_with(
            Some(Layout::PerPackage(vec!["packages".to_string()])),
            "",
            &["packages/parser/src/a.ts"],
            &["packages/parser/changelog.d/2026-10-05-change-a.md"],
        );
        let (code, lines) = verdict(Path::new("/repo"), &facts);
        assert_eq!(code, 0);
        assert_eq!(
            lines,
            vec!["Every scope that changed public surface added its fragments."]
        );
    }

    #[test]
    fn an_unpaid_scope_fails_with_one_annotation_per_finding() {
        let facts = facts_with(
            Some(Layout::PerPackage(vec!["packages".to_string()])),
            "",
            &["packages/parser/src/a.ts"],
            &[],
        );
        let (code, lines) = verdict(Path::new("/repo"), &facts);
        assert_eq!(code, 1);
        // One line per finding, each already rendered as an annotation.
        let found = findings(
            facts.layout.as_ref().unwrap(),
            has_breaking_line(&facts.bodies),
            &facts.changed,
            &facts.added,
        );
        assert_eq!(lines.len(), found.len());
        assert!(lines.iter().all(|line| line.starts_with("::error")));
    }

    #[test]
    fn a_malformed_fragment_name_is_a_finding_even_where_nothing_is_owed() {
        // `changelog.d/` is exempt from `code_touched`, so the only finding can be the name.
        let facts = facts_with(
            Some(Layout::PerPackage(vec!["packages".to_string()])),
            "",
            &["packages/parser/changelog.d/Nope.md"],
            &[],
        );
        let found = findings(
            facts.layout.as_ref().unwrap(),
            has_breaking_line(&facts.bodies),
            &facts.changed,
            &facts.added,
        );
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].file.as_deref(),
            Some("packages/parser/changelog.d/Nope.md")
        );
        assert!(found[0].message.starts_with("fragment filenames are"));
    }

    #[test]
    fn a_scope_owes_only_where_it_is_in_a_container_and_its_code_changed() {
        let containers = Some(Layout::PerPackage(vec!["packages".to_string()]));
        let owed_for = |changed: &[&str]| {
            let facts = facts_with(containers.clone(), "", changed, &[]);
            findings(
                facts.layout.as_ref().unwrap(),
                has_breaking_line(&facts.bodies),
                &facts.changed,
                &facts.added,
            )
        };
        // In a container, but only its own fragment directory moved: nothing owed.
        assert!(owed_for(&["packages/parser/changelog.d/2026-10-05-ok.md"]).is_empty());
        // Code changed, but outside every declared container: nothing owed.
        assert!(owed_for(&["internals/checks/src/a.py"]).is_empty());
        // Both together is the only case that owes.
        assert_eq!(owed_for(&["packages/parser/src/a.ts"]).len(), 1);
    }

    #[test]
    fn a_finding_with_a_file_annotates_that_file() {
        let finding = Finding {
            file: Some("packages/rust/src/lib.rs".into()),
            message: "changed public surface".into(),
        };
        assert_eq!(
            annotation(&finding),
            "::error file=packages/rust/src/lib.rs::changed public surface"
        );
    }

    #[test]
    fn a_finding_with_no_file_annotates_the_workflow() {
        let finding = Finding {
            file: None,
            message: "changed public surface".into(),
        };
        assert_eq!(annotation(&finding), "::error::changed public surface");
    }

    #[test]
    fn owed_names_the_scope_kind_and_fragment_directory() {
        let finding = owed(
            "packages/parser ",
            "packages/parser/changelog.d",
            "changelog",
        );
        assert_eq!(finding.file, None);
        assert!(finding
            .message
            .contains("packages/parser changed public surface"));
        assert!(finding
            .message
            .contains("packages/parser/changelog.d/YYYY-MM-DD-<slug>.md"));
        assert!(finding.message.contains("skip-changelog: <reason>"));
    }

    #[test]
    fn fragment_recognizes_only_a_package_fragment_at_the_expected_depth() {
        let layout = Layout::PerPackage(vec!["packages".to_string()]);
        let found = fragment("packages/parser/changelog.d/2026-09-26-change.md", &layout).unwrap();
        assert_eq!(found.pkg.as_deref(), Some("packages/parser"));
        assert_eq!(found.kind, "changelog");
        assert_eq!(found.name, "2026-09-26-change.md");
        assert!(fragment("other/parser/changelog.d/2026-09-26-change.md", &layout).is_none());
        assert!(fragment("packages/parser/changelog.d/nested/change.md", &layout).is_none());
    }

    #[test]
    fn kind_of_accepts_only_the_two_fragment_directories() {
        assert_eq!(kind_of("changelog.d"), Some("changelog"));
        assert_eq!(kind_of("migrations.d"), Some("migrations"));
        assert_eq!(kind_of("notes.d"), None);
        assert_eq!(kind_of("changelog"), None);
    }

    #[test]
    fn malformed_reports_entries_but_not_fragment_readmes() {
        let layout = Layout::Pooled(Vec::new());
        let changed = vec![
            "changelog.d/README.md".to_string(),
            "changelog.d/2026-09-26-valid.md".to_string(),
            "migrations.d/bad-name.md".to_string(),
            "src/bad-name.md".to_string(),
        ];
        assert_eq!(
            malformed(&layout, &changed),
            vec!["migrations.d/bad-name.md"]
        );
    }

    #[test]
    fn missing_kinds_requires_added_valid_fragments_for_the_same_package() {
        let layout = Layout::PerPackage(vec!["packages".to_string()]);
        let added = vec![
            "packages/parser/changelog.d/2026-09-26-change.md".to_string(),
            "packages/parser/migrations.d/bad-name.md".to_string(),
            "packages/other/migrations.d/2026-09-26-change.md".to_string(),
        ];
        assert_eq!(
            missing_kinds(&layout, &added, Some("packages/parser"), true),
            vec!["migrations"]
        );
        assert!(missing_kinds(&layout, &added, Some("packages/parser"), false).is_empty());
    }

    #[test]
    fn code_touched_ignores_fragments_and_tests_inside_the_package() {
        let pkg = "packages/parser";
        let exempt = vec![
            "packages/parser/changelog.d/2026-09-26-change.md".to_string(),
            "packages/parser/tests/parser.rs".to_string(),
            "packages/other/src/parser.rs".to_string(),
        ];
        assert!(!code_touched(&exempt, pkg));
        assert!(code_touched(
            &["packages/parser/src/parser.rs".to_string()],
            pkg
        ));
    }

    #[test]
    fn exempt_at_any_boundary_checks_nested_test_and_fragment_paths() {
        assert!(exempt_at_any_boundary("packages/parser/tests/parser.rs"));
        assert!(exempt_at_any_boundary(
            "packages/parser/changelog.d/2026-09-26-change.md"
        ));
        assert!(!exempt_at_any_boundary("packages/parser/src/parser.rs"));
    }

    #[test]
    fn exempt_shape_recognizes_archives_attestations_and_test_paths() {
        assert!(exempt_shape("CHANGELOG.md"));
        assert!(exempt_shape("MIGRATIONS.md"));
        assert!(exempt_shape("e2e-attestations/run.json"));
        assert!(exempt_shape("tests/parser.rs"));
        assert!(exempt_shape("src/parser_test.py"));
        assert!(!exempt_shape("src/parser.rs"));
    }

    #[test]
    fn is_test_or_spec_matches_supported_extensions_only() {
        assert!(is_test_or_spec("src/parser.test.rs"));
        assert!(is_test_or_spec("src/parser.spec.tsx"));
        assert!(!is_test_or_spec("src/parser.test.txt"));
        assert!(!is_test_or_spec("src/parser.rs"));
    }

    #[test]
    fn fragment_dirs_finds_shallow_directories_and_skips_build_trees() {
        let root = std::env::temp_dir().join(format!("tc-changelog-inline-{}", std::process::id()));
        std::fs::create_dir_all(root.join("packages/parser/changelog.d")).unwrap();
        std::fs::create_dir_all(root.join("target/debug/migrations.d")).unwrap();
        std::fs::create_dir_all(root.join("packages/parser/deep/migrations.d")).unwrap();
        let dirs = fragment_dirs(&root);
        assert_eq!(
            dirs,
            vec![vec![
                "packages".to_string(),
                "parser".to_string(),
                "changelog.d".to_string()
            ]]
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A tree under a directory unique to `slug` and this process, for a test that reads the
    /// filesystem.
    fn tree(slug: &str) -> std::path::PathBuf {
        let root =
            std::env::temp_dir().join(format!("tc-changelog-inline-{slug}-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    fn touch(root: &std::path::Path, rel: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "").unwrap();
    }

    #[test]
    fn package_roots_are_the_manifest_holding_directories_inside_a_container() {
        let root = tree("roots");
        touch(&root, "packages/parser/package.json");
        touch(&root, "crates/lexer/Cargo.toml");
        touch(&root, "services/engine/pyproject.toml");
        touch(&root, "docs/guide.md");
        // A manifest at the root is not a package sitting inside a container directory.
        touch(&root, "package.json");
        assert_eq!(
            package_roots(&root),
            vec![
                "crates/lexer".to_string(),
                "packages/parser".to_string(),
                "services/engine".to_string(),
            ]
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_package_walk_skips_dependency_and_build_directories() {
        let root = tree("skips");
        touch(&root, "node_modules/left-pad/package.json");
        touch(&root, "target/debug/Cargo.toml");
        touch(&root, ".git/modules/x/package.json");
        touch(&root, "packages/node_modules/package.json");
        assert_eq!(package_roots(&root), Vec::<String>::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_directory_the_walk_cannot_read_holds_no_packages() {
        // An unreadable or absent container is not a container of packages.
        let root = tree("unreadable");
        let missing = root.join("absent");
        assert_eq!(child_dirs(&missing), Vec::<String>::new());
        assert_eq!(packages_in(&missing, "packages"), Vec::<String>::new());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_pooled_layout_carries_the_package_roots_it_discovered() {
        let root = tree("discover");
        std::fs::create_dir_all(root.join("docs/changelog.d")).unwrap();
        touch(&root, "packages/parser/package.json");
        assert_eq!(
            discover_layout(&root),
            Some(Layout::Pooled(vec!["packages/parser".to_string()]))
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn names_package_reads_the_slug_after_the_date() {
        assert!(names_package(
            "2026-09-21-parser-drop-the-flag.md",
            Some("parser")
        ));
        assert!(!names_package(
            "2026-09-21-emitter-drop-the-flag.md",
            Some("parser")
        ));
        // The package name is a whole segment, so a hyphen has to follow it, and the name
        // carries a slug of its own as well as the package.
        assert!(!names_package("2026-09-21-parserdrop.md", Some("parser")));
        assert!(!names_package("2026-09-21-parser.md", Some("parser")));
        // A fragment is a markdown file.
        assert!(!names_package("2026-09-21-parser-x", Some("parser")));
        // No package root means no name for the fragment to carry.
        assert!(names_package("2026-09-21-anything.md", None));
    }

    #[test]
    fn records_reads_the_directory_per_package_and_the_name_when_pooled() {
        let frag = Fragment {
            pkg: Some("packages/parser".to_string()),
            kind: "changelog",
            name: "2026-09-21-emitter-drop-the-flag.md".to_string(),
        };
        // Under the per-package layout the directory names the package, so a filename naming
        // another package cannot leave this one unpaid.
        let per_package = Layout::PerPackage(vec!["packages".to_string()]);
        assert!(records(&per_package, &frag, Some("packages/parser")));
        assert!(!records(&per_package, &frag, Some("packages/emitter")));
        let pooled = Layout::Pooled(vec!["packages/parser".to_string()]);
        assert!(!records(&pooled, &frag, Some("parser")));
        assert!(records(&pooled, &frag, Some("emitter")));
    }

    #[test]
    fn pooled_scopes_fall_back_to_the_whole_tree() {
        let roots = vec!["packages/parser".to_string(), "crates/lexer".to_string()];
        assert_eq!(
            pooled_scopes(&roots),
            vec![Some("packages/parser"), Some("crates/lexer")]
        );
        assert_eq!(pooled_scopes(&[]), vec![None]);
    }

    #[test]
    fn pooled_surface_is_the_package_root_or_every_non_exempt_path() {
        let changed = vec!["packages/parser/src/lex.py".to_string()];
        assert!(pooled_surface_touched(&changed, Some("packages/parser")));
        assert!(!pooled_surface_touched(&changed, Some("packages/emitter")));
        assert!(pooled_surface_touched(&changed, None));
        let exempt = vec!["packages/parser/src/lex_test.py".to_string()];
        assert!(!pooled_surface_touched(&exempt, None));
    }

    #[test]
    fn package_name_is_the_last_segment_of_the_root() {
        assert_eq!(package_name("packages/parser"), "parser");
        assert_eq!(package_name("parser"), "parser");
    }

    #[test]
    fn owed_pooled_names_the_package_in_the_fragment_it_asks_for() {
        let with_root = owed_pooled(Some("packages/parser"), Some("parser"), "migrations");
        assert_eq!(with_root.file, None);
        assert!(with_root.message.starts_with("packages/parser marked"));
        assert!(with_root
            .message
            .contains("migrations.d/YYYY-MM-DD-parser-<slug>.md"));
        let whole_tree = owed_pooled(None, None, "changelog");
        assert!(whole_tree.message.starts_with("changed public surface"));
        assert!(whole_tree
            .message
            .contains("changelog.d/YYYY-MM-DD-<slug>.md"));
    }

    #[test]
    fn per_package_findings_asks_a_package_in_a_container_that_changed_code() {
        let layout = Layout::PerPackage(vec!["packages".to_string()]);
        let containers = vec!["packages".to_string()];
        let changed = vec!["packages/parser/src/lex.py".to_string()];
        assert_eq!(
            per_package_findings(&layout, &containers, false, &changed, &[]).len(),
            1
        );
    }

    #[test]
    fn per_package_findings_skips_a_package_outside_every_container() {
        let layout = Layout::PerPackage(vec!["packages".to_string()]);
        let containers = vec!["packages".to_string()];
        let changed = vec!["vendor/parser/src/lex.py".to_string()];
        assert_eq!(
            per_package_findings(&layout, &containers, false, &changed, &[]),
            vec![]
        );
    }

    #[test]
    fn per_package_findings_skips_a_package_whose_changed_paths_are_all_exempt() {
        let layout = Layout::PerPackage(vec!["packages".to_string()]);
        let containers = vec!["packages".to_string()];
        let changed = vec!["packages/parser/src/lex_test.py".to_string()];
        assert_eq!(
            per_package_findings(&layout, &containers, false, &changed, &[]),
            vec![]
        );
    }

    #[test]
    fn pooled_findings_scope_each_package_root_separately() {
        let layout = Layout::Pooled(vec![
            "packages/emitter".to_string(),
            "packages/parser".to_string(),
        ]);
        let changed = vec![
            "packages/parser/src/lex.py".to_string(),
            "packages/emitter/src/emit.py".to_string(),
            "README.md".to_string(),
        ];
        let added = vec!["docs/changelog.d/2026-09-21-parser-drop-the-flag.md".to_string()];
        let found = findings(&layout, false, &changed, &added);
        assert_eq!(found.len(), 1);
        assert!(found[0].message.starts_with("packages/emitter changed"));
    }
}
