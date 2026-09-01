//! `e2e attest` / `e2e verify` — the e2e decision nudge. `attest` records the runner's chosen
//! command as a branch-keyed receipt; `verify` confirms a branch changing scoped source has one.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Where the branch-keyed receipts live, relative to the package root: `<branch_slug>.json`.
pub const RECEIPTS_DIR: &str = "e2e-attestations";

/// The retired single-file attestation location: never a receipt, never scoped source.
const LEGACY_ATTESTATION: &str = "e2e-attestation.json";

/// A record of one e2e decision, written to `RECEIPTS_DIR/<branch_slug>.json`. Everything
/// here is for humans — [`verify`] reads only the receipt's presence in the branch's diff.
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
    /// The raw branch name the receipt is keyed by; the filename carries only its slug.
    #[serde(default)]
    pub branch: String,
}

/// The standardized receipt slug for a branch name — the receipt lives at
/// `e2e-attestations/<slug>.json`. Lowercased; every character outside `[a-z0-9._-]` becomes
/// `-`; runs collapse; truncated to 80; edges trimmed; an empty result falls back to `branch`.
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

/// The checked-out branch of `repo`; a detached HEAD is an error naming the fix.
pub(crate) fn current_branch(repo: &Path) -> Result<String> {
    git_capture(repo, &["symbolic-ref", "--short", "-q", "HEAD"]).context(
        "resolving the current branch — the receipt is keyed by branch, so this \
         must run on a checked-out branch (a detached HEAD has none): `git switch <branch>`",
    )
}

/// Run `command` in `repo` and, when it passes, write and commit the branch's receipt at
/// `repo`/[`RECEIPTS_DIR`]`/<branch_slug>.json`. A non-zero `command` leaves the receipts
/// untouched; the returned [`Attestation::exit_code`] carries the failure either way.
pub fn attest(repo: &Path, command: &str) -> Result<Attestation> {
    let commit = git_capture(repo, &["rev-parse", "HEAD"])
        .context("resolving HEAD — `e2e attest` must run inside a git repo with a commit")?;
    let branch = current_branch(repo)?;

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

    if exit_code != 0 {
        return Ok(attestation);
    }

    // Only ever add: a paired delete reads as a rename to git and conflicts across parallel
    // branches — `docs/explanation/e2e.md`.
    let dir = repo.join(RECEIPTS_DIR);
    std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
    let path = dir.join(format!("{}.json", branch_slug(&branch)));
    let json = serde_json::to_string_pretty(&attestation).context("serializing the receipt")?;
    std::fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("writing {}", path.display()))?;
    git_run(repo, &["add", "-A", "--", RECEIPTS_DIR])?;

    let message = format!("e2e attestation for {branch}");
    // A plain commit inherits the repo's signing policy, so a repo requiring verified
    // signatures gets a signed (mergeable) receipt.
    git_run(repo, &["commit", "-q", "-m", message.as_str()])?;

    Ok(attestation)
}

/// The outcome of [`verify`] — whether a committed receipt answers the branch's e2e nudge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verification {
    /// The branch owes no decision, or a receipt in its diff answers the one it owes.
    Fresh,
    /// No receipt answers the nudge — the gate fails.
    Missing,
}

/// Verify the e2e decision at `repo` — the CI side of the nudge. Equivalent to
/// [`verify_scoped`] with `scope` set to `repo`.
pub fn verify(repo: &Path) -> Result<Verification> {
    verify_scoped(repo, repo)
}

/// Verify the e2e decision at `repo`, with `scope` (rather than all of `repo`) defining what
/// counts as scoped source; `scope` must be `repo` or a descendant. Equivalent to
/// [`verify_since`] with no `base`.
pub fn verify_scoped(repo: &Path, scope: &Path) -> Result<Verification> {
    verify_since(repo, scope, None)
}

/// Equivalent to [`verify_extra_scoped`] with no extra roots and no excludes.
pub fn verify_since(repo: &Path, scope: &Path, base: Option<&str>) -> Result<Verification> {
    verify_extra_scoped(repo, scope, base, &[], &[])
}

/// Verify the e2e decision at `repo`, joining **extra scopes** outside `scope` into what
/// counts as scoped source and subtracting `excludes`. With `base`, both checks are content
/// diffs of `<base>...HEAD`; without one, a committed receipt at `repo` is the whole check.
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

    // Question 1 — did this branch change the scoped source?
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
    // A receipt anywhere in the tree — a monorepo sibling's, an extra scope's — is not scoped
    // source either.
    args.push(format!(":(top,exclude,glob)**/{RECEIPTS_DIR}/**"));
    args.push(format!(":(top,exclude,glob)**/{LEGACY_ATTESTATION}"));
    for exclude in excludes {
        args.push(format!(":(top,exclude){}", exclude.display()));
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    if !git_diff_changed(repo, &arg_refs)? {
        return Ok(Verification::Fresh);
    }

    // Question 2 — does the branch's diff add or update a receipt? The filter drops
    // deletions, so sweeping a stale receipt by hand never counts as a decision.
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

/// `scope` as a pathspec relative to `repo` — git resolves pathspecs against the invocation's
/// cwd, which is always `repo` here. `.` when `scope` is `repo` itself.
fn relative_pathspec(repo: &Path, scope: &Path) -> String {
    if scope == repo {
        return ".".to_string();
    }
    match scope.strip_prefix(repo) {
        Ok(rel) if !rel.as_os_str().is_empty() => rel.to_string_lossy().into_owned(),
        _ => scope.to_string_lossy().into_owned(),
    }
}

/// Confirm `scope` and every `extra_scope` name at least one path git tracks under `repo`.
/// A pathspec matching nothing diffs to empty forever, so a typo'd scope would wave every
/// branch through; erroring names the bad scope instead.
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

/// `true` when git tracks at least one path matching `pathspec` (run with cwd `repo`). A
/// pathspec git rejects as outside the repository counts as "matches nothing".
fn pathspec_matches_tracked(repo: &Path, pathspec: &str) -> Result<bool> {
    let out = Command::new("git")
        .args(["ls-files", "--", pathspec])
        .current_dir(repo)
        .output()
        .with_context(|| format!("running `git ls-files -- {pathspec}`"))?;
    Ok(out.status.success() && !out.stdout.is_empty())
}

/// Run `git diff --quiet …` in `repo`: `false` for no differences, `true` for differences, an
/// error for anything else — a bad base ref must fail loudly, never read as "no changes".
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
