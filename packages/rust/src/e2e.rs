//! `e2e attest` / `e2e verify` — the e2e decision nudge.
//!
//! `attest` runs the e2e command of the runner's choosing and records the
//! decision as a branch-keyed receipt; `verify` confirms in CI that a branch
//! changing the scoped source carries a receipt in its own diff. CI never runs
//! e2e, and the command is unrestricted — the choice of command (the full
//! suite, a targeted subset, a no-op) *is* the judgment the receipt records.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Where the branch-keyed receipts live, relative to the package root. Each
/// receipt is `<branch_slug>.json`, so parallel branches write distinct files.
pub const RECEIPTS_DIR: &str = "e2e-attestations";

/// The retired single-file attestation location: never read as a receipt and
/// never counted as scoped source, so a branch deleting it owes nothing.
const LEGACY_ATTESTATION: &str = "e2e-attestation.json";

/// A record of one e2e decision — written to `RECEIPTS_DIR/<branch_slug>.json`
/// and committed by [`attest`].
///
/// Everything here is information for humans — [`verify`] reads only the
/// receipt's presence in the branch's diff, never its contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    /// The command that was run (e.g. `pnpm run e2e`) — the judgment itself.
    pub command: String,
    /// When it ran, as a Unix timestamp (seconds).
    pub ran_at: u64,
    /// The command's exit code — recorded, never gated on.
    pub exit_code: i32,
    /// The commit the run was made against (HEAD at attest time).
    pub commit: String,
    /// The raw branch name the receipt is keyed by (the filename carries only
    /// its sanitized slug).
    #[serde(default)]
    pub branch: String,
}

/// The standardized receipt slug for a branch name — the receipt lives at
/// `e2e-attestations/<slug>.json`. Lowercased; every character outside
/// `[a-z0-9._-]` becomes `-`; runs of `-` collapse to one; truncated to 80
/// characters; leading/trailing `-` and `.` trimmed; an empty result falls
/// back to `branch`. Deterministic and git-free, so a script can locate a
/// branch's receipt; exposed on the CLI as `e2e slug`.
pub fn branch_slug(branch: &str) -> String {
    let mut slug = String::new();
    for c in branch.to_lowercase().chars() {
        let mapped = if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' {
            c
        } else {
            '-'
        };
        if mapped == '-' && slug.ends_with('-') {
            continue;
        }
        slug.push(mapped);
    }
    let slug: String = slug.chars().take(80).collect();
    let slug = slug.trim_matches(|c| c == '-' || c == '.');
    if slug.is_empty() {
        "branch".to_string()
    } else {
        slug.to_string()
    }
}

/// The checked-out branch of `repo`, or an error naming the fix on a detached
/// HEAD — the receipt is keyed by branch, so attest needs one.
pub(crate) fn current_branch(repo: &Path) -> Result<String> {
    git_capture(repo, &["symbolic-ref", "--short", "-q", "HEAD"]).context(
        "resolving the current branch — the receipt is keyed by branch, so this \
         must run on a checked-out branch (a detached HEAD has none): `git switch <branch>`",
    )
}

/// Run `command` in `repo`, write the branch's receipt to
/// `repo`/[`RECEIPTS_DIR`]`/<branch_slug>.json`, prune the receipts other
/// branches left behind (and the retired single-file attestation), and commit.
/// Returns the attestation.
///
/// Writes regardless of the command's exit code — the record is the decision
/// and what ran, and the honest result is part of the record.
pub fn attest(repo: &Path, command: &str) -> Result<Attestation> {
    let commit = git_capture(repo, &["rev-parse", "HEAD"])
        .context("resolving HEAD — `e2e attest` must run inside a git repo with a commit")?;
    let branch = current_branch(repo)?;

    // Run the e2e command via the shell, streaming its output through.
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(repo)
        .status()
        .with_context(|| format!("running e2e command `{command}`"))?;
    let exit_code = status.code().unwrap_or(-1);

    let ran_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let attestation = Attestation {
        command: command.to_string(),
        ran_at,
        exit_code,
        commit,
        branch: branch.clone(),
    };

    // Prune sibling receipts — dead weight once their branches merge, since
    // `verify` reads only the current branch's diff. Write-once files make the
    // deletions merge-clean (delete/delete or delete/absent, never a conflict).
    let dir = repo.join(RECEIPTS_DIR);
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    for entry in std::fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "json") {
            std::fs::remove_file(&path).with_context(|| format!("pruning {}", path.display()))?;
        }
    }
    let path = dir.join(format!("{}.json", branch_slug(&branch)));
    let json = serde_json::to_string_pretty(&attestation).context("serializing the receipt")?;
    std::fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("writing {}", path.display()))?;
    git_run(repo, &["add", "-A", "--", RECEIPTS_DIR])?;

    // The retired single-file attestation is dead weight too; collect it here
    // so the migration is one attest away.
    if pathspec_matches_tracked(repo, LEGACY_ATTESTATION)? {
        git_run(
            repo,
            &["rm", "-q", "--ignore-unmatch", "--", LEGACY_ATTESTATION],
        )?;
    }

    let message = format!("e2e attestation for {branch}");
    // A plain commit that inherits the repo's signing policy: a repo requiring
    // verified signatures gets a signed (mergeable) receipt, instead of the
    // unsigned commit a forced `commit.gpgsign=false` would leave behind.
    git_run(repo, &["commit", "-q", "-m", message.as_str()])?;

    Ok(attestation)
}

/// The outcome of [`verify`] — whether a committed receipt answers the branch's
/// e2e nudge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verification {
    /// The branch owes no decision (its scoped diff is empty), or a receipt in
    /// its diff answers the one it owes — the gate passes.
    Fresh,
    /// No receipt answers the nudge: the branch changed the scoped source and
    /// its diff adds or updates no receipt (or, without a base, no committed
    /// receipt is present at all) — the gate fails.
    Missing,
}

/// Verify the e2e decision at `repo` — the CI side of the nudge. Reads only
/// receipt presence and content diffs: never runs e2e, never inspects a
/// recorded command or exit code, never compares commit SHAs.
///
/// Equivalent to [`verify_scoped`] with `scope` set to `repo`.
pub fn verify(repo: &Path) -> Result<Verification> {
    verify_scoped(repo, repo)
}

/// Verify the e2e decision at `repo`, with `scope` (rather than all of `repo`)
/// defining what counts as scoped source.
///
/// `repo` and `scope` serve different roles: `repo` is where the receipts live
/// (the package root — a manifest's own natural home), while `scope` is what
/// counts as "code" (the directory a `path`-scoped call actually named, which
/// can be narrower — a package root commonly also holds `tests/`, docs, and
/// config files that aren't the attestable source). `scope` must be `repo` or
/// a descendant of it.
///
/// Equivalent to [`verify_since`] with no `base`.
pub fn verify_scoped(repo: &Path, scope: &Path) -> Result<Verification> {
    verify_since(repo, scope, None)
}

/// Equivalent to [`verify_extra_scoped`] with no extra roots and no excludes.
pub fn verify_since(repo: &Path, scope: &Path, base: Option<&str>) -> Result<Verification> {
    verify_extra_scoped(repo, scope, base, &[], &[])
}

/// Verify the e2e decision at `repo`, joining **extra scopes** outside `scope`
/// into what counts as scoped source.
///
/// With `base`, both checks are content diffs of `<base>...HEAD`, read from
/// the merge base — indifferent to commit identity, so rebases and squash
/// merges never disturb a receipt:
///
/// 1. A branch whose diff leaves the scoped source untouched owes no decision
///    and passes. The scoped source is the union of `scope` and every
///    repo-root-relative `extra_scopes` entry (a shared source tree beside the
///    package — a native core bound into several bindings — which no `scope`
///    at-or-below `repo` can reach), minus the `excludes` (feature-gated
///    subtrees compiled out of the package). Receipts and the retired
///    single-file attestation are never scoped source.
/// 2. Otherwise the branch passes when its diff **adds or updates** a receipt
///    under `repo`'s receipts directory. A deletion (the prune) is not a
///    decision.
///
/// Without `base` there is no branch diff to read, so presence is the check: a
/// committed receipt at `repo` passes.
pub fn verify_extra_scoped(
    repo: &Path,
    scope: &Path,
    base: Option<&str>,
    extra_scopes: &[PathBuf],
    excludes: &[PathBuf],
) -> Result<Verification> {
    let Some(base) = base else {
        return Ok(if has_receipts(repo) {
            Verification::Fresh
        } else {
            Verification::Missing
        });
    };
    validate_scopes(repo, scope, extra_scopes)?;

    // Question 1 — did this branch change the scoped source? `<base>...HEAD`
    // diffs from the merge base, so only the branch's own changes count.
    let mut args: Vec<String> = vec![
        "diff".into(),
        "--quiet".into(),
        format!("{base}...HEAD"),
        "--".into(),
        relative_pathspec(repo, scope),
    ];
    for extra in extra_scopes {
        args.push(format!(":(top){}", extra.display()));
    }
    args.push(format!(":(exclude){RECEIPTS_DIR}"));
    args.push(format!(":(exclude){LEGACY_ATTESTATION}"));
    // Receipts and legacy files anywhere in the tree (a monorepo sibling's, an
    // extra scope's) are never scoped source either.
    args.push(format!(":(top,exclude,glob)**/{RECEIPTS_DIR}/**"));
    args.push(format!(":(top,exclude,glob)**/{LEGACY_ATTESTATION}"));
    for exclude in excludes {
        args.push(format!(":(top,exclude){}", exclude.display()));
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    if !git_diff_changed(repo, &arg_refs)? {
        return Ok(Verification::Fresh);
    }

    // Question 2 — does the branch's diff add or update a receipt? The
    // diff-filter drops deletions, so the prune never counts as a decision.
    let out = git_capture(
        repo,
        &[
            "diff",
            "--name-only",
            "--diff-filter=ACMRT",
            &format!("{base}...HEAD"),
            "--",
            RECEIPTS_DIR,
        ],
    )?;
    Ok(if out.is_empty() {
        Verification::Missing
    } else {
        Verification::Fresh
    })
}

/// `true` when a receipt (`*.json` under [`RECEIPTS_DIR`]) sits at `repo`.
fn has_receipts(repo: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(repo.join(RECEIPTS_DIR)) else {
        return false;
    };
    entries
        .flatten()
        .any(|e| e.path().extension().is_some_and(|ext| ext == "json") && e.path().is_file())
}

/// `scope` as a pathspec relative to `repo` (git resolves pathspecs relative to
/// the invocation's cwd, which is always `repo` here). `.` when `scope` is
/// `repo` itself.
fn relative_pathspec(repo: &Path, scope: &Path) -> String {
    if scope == repo {
        return ".".to_string();
    }
    match scope.strip_prefix(repo) {
        Ok(rel) if !rel.as_os_str().is_empty() => rel.to_string_lossy().into_owned(),
        _ => scope.to_string_lossy().into_owned(),
    }
}

/// Confirm `scope` and every `extra_scope` name at least one path git tracks under
/// `repo`, erroring loudly on one that matches nothing (#391).
///
/// A typo'd or outside `scope` otherwise falls through [`relative_pathspec`] as a
/// pathspec matching nothing, and a diff over nothing is always empty — a branch
/// that changed real source would pass forever. Each `extra_scope` has the same
/// failure mode: a misspelled shared-tree root silently drops out of the scoped
/// diff. Confirming the pathspec matches a tracked path first turns both into an
/// honest error naming the bad scope.
fn validate_scopes(repo: &Path, scope: &Path, extra_scopes: &[PathBuf]) -> Result<()> {
    let scope_spec = relative_pathspec(repo, scope);
    if !pathspec_matches_tracked(repo, &scope_spec)? {
        bail!(
            "e2e verify: --scope `{}` matches no tracked path under `{}` — \
             --scope must name `{}` or a directory beneath it that git tracks",
            scope.display(),
            repo.display(),
            repo.display(),
        );
    }
    for extra in extra_scopes {
        let extra_spec = format!(":(top){}", extra.display());
        if !pathspec_matches_tracked(repo, &extra_spec)? {
            bail!(
                "e2e verify: --extra-scope `{}` matches no tracked path — \
                 --extra-scope must name a repo-root-relative directory that git tracks",
                extra.display(),
            );
        }
    }
    Ok(())
}

/// `true` when git tracks at least one path matching `pathspec` (run with cwd
/// `repo`). A pathspec git rejects as outside the repository exits non-zero; that
/// is treated identically to "matches nothing" — either way the scope names no
/// tracked path.
fn pathspec_matches_tracked(repo: &Path, pathspec: &str) -> Result<bool> {
    let out = Command::new("git")
        .args(["ls-files", "--", pathspec])
        .current_dir(repo)
        .output()
        .with_context(|| format!("running `git ls-files -- {pathspec}`"))?;
    Ok(out.status.success() && !out.stdout.is_empty())
}

/// Run `git diff --quiet …` in `repo`: `false` for no differences, `true` for
/// differences, an error (with git's stderr) for anything else — a bad base
/// ref must fail loudly, never read as "no changes".
fn git_diff_changed(repo: &Path, args: &[&str]) -> Result<bool> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("running `git {}`", args.join(" ")))?;
    match out.status.code() {
        Some(0) => Ok(false),
        Some(1) => Ok(true),
        _ => bail!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ),
    }
}

/// Run `git` with `args` in `repo`, returning trimmed stdout; errors if git fails.
fn git_capture(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("running `git {}`", args.join(" ")))?;
    if !out.status.success() {
        bail!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8(out.stdout)?.trim().to_string())
}

/// Run `git` with `args` in `repo` for its side effect; errors if git fails.
fn git_run(repo: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .with_context(|| format!("running `git {}`", args.join(" ")))?;
    if !status.success() {
        bail!("`git {}` failed", args.join(" "));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::branch_slug;

    #[test]
    fn slug_lowercases_and_maps_separators() {
        assert_eq!(branch_slug("feature/one"), "feature-one");
        assert_eq!(branch_slug("Feature/One"), "feature-one");
        assert_eq!(
            branch_slug("claude/e2e-attestation-conflicts-mrkc1b"),
            "claude-e2e-attestation-conflicts-mrkc1b"
        );
    }

    #[test]
    fn slug_keeps_dots_and_underscores() {
        assert_eq!(branch_slug("v1.2_rc"), "v1.2_rc");
    }

    #[test]
    fn slug_collapses_runs_and_trims_edges() {
        assert_eq!(branch_slug("wip//Émil's"), "wip-mil-s");
        assert_eq!(branch_slug("--dashes--"), "dashes");
        assert_eq!(branch_slug(".hidden."), "hidden");
    }

    #[test]
    fn slug_truncates_to_80() {
        let long = "x".repeat(300);
        assert_eq!(branch_slug(&long).len(), 80);
    }

    #[test]
    fn slug_never_returns_empty() {
        assert_eq!(branch_slug(""), "branch");
        assert_eq!(branch_slug("É"), "branch");
    }
}
