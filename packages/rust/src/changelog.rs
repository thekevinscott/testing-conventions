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
    migrations: bool,
}

/// Read the repository facts the verdict decides on.
fn facts(root: &Path, base: &str) -> Result<Facts> {
    Ok(Facts {
        layout: discover_layout(root),
        bodies: commit_bodies(root, base)?,
        changed: changed_files(root, base)?,
        added: added_files(root, base)?,
        migrations: migrations_enforced(root),
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
    let found = findings(layout, facts.migrations, &facts.changed, &facts.added);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn facts_with(layout: Option<Layout>, bodies: &str, changed: &[&str], added: &[&str]) -> Facts {
        Facts {
            layout,
            bodies: bodies.to_string(),
            changed: changed.iter().map(|s| s.to_string()).collect(),
            added: added.iter().map(|s| s.to_string()).collect(),
            migrations: false,
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
            facts.migrations,
            &facts.changed,
            &facts.added,
        );
        assert_eq!(lines.len(), found.len());
        assert!(lines.iter().all(|line| line.starts_with("::error")));
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
        let layout = Layout::Pooled;
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
}
