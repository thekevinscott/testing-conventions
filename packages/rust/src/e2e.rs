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

    let status = run_shell(repo, command)?;
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

/// Run `command` through `sh -c` in `repo`, returning its exit status.
fn run_shell(repo: &Path, command: &str) -> Result<std::process::ExitStatus> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(repo)
        .status()
        .with_context(|| format!("running e2e command `{command}`"))
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

/// Equivalent to [`verify_extra_scoped`] with no extra roots, no excludes, and no `branch`
/// override — Question 2 reads the checked-out branch's own receipt.
pub fn verify_since(repo: &Path, scope: &Path, base: Option<&str>) -> Result<Verification> {
    verify_extra_scoped(repo, scope, base, &[], &[], None)
}

/// Verify the e2e decision at `repo`, joining **extra scopes** outside `scope` into what
/// counts as scoped source and subtracting `excludes`. With `base`, Question 2 scopes to
/// `branch`'s own receipt (the checked-out branch if absent); without `base`, any receipt suffices.
pub fn verify_extra_scoped(
    repo: &Path,
    scope: &Path,
    base: Option<&str>,
    extra_scopes: &[PathBuf],
    excludes: &[PathBuf],
    branch: Option<&str>,
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
    let args = scoped_source_args(repo, scope, base, extra_scopes, excludes);
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    if !git_diff_changed(repo, &arg_refs)? {
        return Ok(Verification::Fresh);
    }

    // Question 2 — does the acting branch's diff add or update *its own* receipt?
    receipt_freshness(repo, base, branch)
}

/// Question 2: `Fresh` when `base...HEAD` adds or updates the acting branch's own receipt.
/// The acting branch is `branch`, else the checked-out one, else none — and with none to
/// name, every receipt counts. `--diff-filter=ACMRT` drops deletions, so sweeping a stale
/// receipt by hand never passes as freshness.
fn receipt_freshness(repo: &Path, base: &str, branch: Option<&str>) -> Result<Verification> {
    let range = format!("{base}...HEAD");
    let acting_branch = branch
        .map(str::to_string)
        .or_else(|| current_branch(repo).ok());
    let pathspec = receipt_pathspec(acting_branch.as_deref());
    let receipt_diff = [
        "diff",
        "--name-only",
        "--diff-filter=ACMRT",
        &range,
        "--",
        &pathspec,
    ];
    let out = git_capture(repo, &receipt_diff)?;
    Ok(if out.is_empty() {
        Verification::Missing
    } else {
        Verification::Fresh
    })
}

/// The full `git diff --quiet` argv asking whether `base...HEAD` touched scoped source:
/// `scope` plus every extra scope, minus `excludes` and minus every receipt path anywhere in
/// the tree — a receipt is never scoped source, not even a monorepo sibling's.
fn scoped_source_args(
    repo: &Path,
    scope: &Path,
    base: &str,
    extra_scopes: &[PathBuf],
    excludes: &[PathBuf],
) -> Vec<String> {
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
    args.push(format!(":(top,exclude,glob)**/{RECEIPTS_DIR}/**"));
    args.push(format!(":(top,exclude,glob)**/{LEGACY_ATTESTATION}"));
    for exclude in excludes {
        args.push(format!(":(top,exclude){}", exclude.display()));
    }
    args
}

/// The pathspec Question 2 counts receipts under: the acting branch's own receipt file when
/// there is a branch to name, and otherwise the whole receipts directory, so every receipt
/// counts.
fn receipt_pathspec(acting_branch: Option<&str>) -> String {
    match acting_branch {
        Some(branch) => format!("{RECEIPTS_DIR}/{}.json", branch_slug(branch)),
        None => RECEIPTS_DIR.to_string(),
    }
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
        bail!("{}", untracked_scope(repo, scope));
    }
    for extra in extra_scopes {
        let extra_spec = format!(":(top){}", extra.display());
        if !pathspec_matches_tracked(repo, &extra_spec)? {
            bail!("{}", untracked_extra_scope(extra));
        }
    }
    Ok(())
}

/// What a `--scope` matching no tracked path reports, naming the scope git found nothing for.
fn untracked_scope(repo: &Path, scope: &Path) -> String {
    format!(
        "e2e verify: --scope `{}` matches no tracked path under `{}` — \
         --scope must name `{}` or a directory beneath it that git tracks",
        scope.display(),
        repo.display(),
        repo.display(),
    )
}

/// What an `--extra-scope` matching no tracked path reports, naming the scope git found nothing
/// for.
fn untracked_extra_scope(extra: &Path) -> String {
    format!(
        "e2e verify: --extra-scope `{}` matches no tracked path — \
         --extra-scope must name a repo-root-relative directory that git tracks",
        extra.display(),
    )
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
    use super::{
        branch_slug, git_capture, git_diff_changed, git_run, pathspec_matches_tracked,
        receipt_pathspec, run_shell, scoped_source_args, untracked_extra_scope, untracked_scope,
        validate_scopes, LEGACY_ATTESTATION, RECEIPTS_DIR,
    };
    use std::path::{Path, PathBuf};

    /// A scratch repository with one file staged under `src/`. `ls-files` reads the index, so
    /// nothing needs committing — and no identity needs configuring.
    fn repo_tracking_src(slug: &str) -> PathBuf {
        let repo = std::env::temp_dir().join(format!("tc-e2e-{slug}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&repo);
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::write(repo.join("src").join("a.rs"), "pub fn f() {}\n").unwrap();
        git_run(&repo, &["init", "-q"]).unwrap();
        git_run(&repo, &["add", "-A", "--", "src"]).unwrap();
        repo
    }

    #[test]
    fn scope_validation_passes_a_tracked_scope_and_rejects_either_untracked_one() {
        let repo = repo_tracking_src("validate-scopes");
        let tracked = repo.join("src");

        validate_scopes(&repo, &tracked, &[PathBuf::from("src")])
            .expect("a tracked scope and a tracked extra scope both pass");

        // A typo'd `--scope` must error rather than diff to empty and wave the branch through.
        let err = format!(
            "{:#}",
            validate_scopes(&repo, &repo.join("nope"), &[]).unwrap_err()
        );
        assert!(err.contains("--scope `"), "got: {err}");
        assert!(err.contains("matches no tracked path under"), "got: {err}");

        // So must a typo'd `--extra-scope`, even behind a good `--scope`.
        let err = format!(
            "{:#}",
            validate_scopes(&repo, &tracked, &[PathBuf::from("nope")]).unwrap_err()
        );
        assert!(err.contains("--extra-scope `nope`"), "got: {err}");

        let _ = std::fs::remove_dir_all(&repo);
    }

    #[test]
    fn an_untracked_scope_names_the_scope_the_repository_and_what_scope_must_be() {
        let message = untracked_scope(Path::new("/repo"), Path::new("/repo/packages/typo"));
        assert_eq!(
            message,
            "e2e verify: --scope `/repo/packages/typo` matches no tracked path under `/repo` — \
             --scope must name `/repo` or a directory beneath it that git tracks"
        );
    }

    #[test]
    fn an_untracked_extra_scope_names_the_scope_and_that_it_is_repo_root_relative() {
        let message = untracked_extra_scope(Path::new("internals/typo"));
        assert_eq!(
            message,
            "e2e verify: --extra-scope `internals/typo` matches no tracked path — \
             --extra-scope must name a repo-root-relative directory that git tracks"
        );
    }

    #[test]
    fn a_named_branch_scopes_question_two_to_its_own_receipt() {
        assert_eq!(
            receipt_pathspec(Some("chore/750-ratchet-rust-step4")),
            format!("{RECEIPTS_DIR}/chore-750-ratchet-rust-step4.json")
        );
    }

    #[test]
    fn with_no_branch_to_name_every_receipt_counts() {
        assert_eq!(receipt_pathspec(None), RECEIPTS_DIR);
    }

    #[test]
    fn the_scoped_source_argv_excludes_every_receipt_path_in_the_tree() {
        let args = scoped_source_args(
            Path::new("/repo"),
            Path::new("/repo/packages/rust"),
            "main",
            &[],
            &[],
        );
        assert!(args.starts_with(&[
            "diff".to_string(),
            "--quiet".to_string(),
            "main...HEAD".to_string(),
            "--".to_string(),
            "packages/rust".to_string(),
        ]));
        // Both the scope-local and the tree-wide form, so a monorepo sibling's receipt is
        // not scoped source either.
        assert!(args.contains(&format!(":(exclude){RECEIPTS_DIR}")));
        assert!(args.contains(&format!(":(top,exclude,glob)**/{RECEIPTS_DIR}/**")));
        assert!(args.contains(&format!(":(exclude){LEGACY_ATTESTATION}")));
        assert!(args.contains(&format!(":(top,exclude,glob)**/{LEGACY_ATTESTATION}")));
    }

    #[test]
    fn extra_scopes_join_the_argv_and_excludes_subtract_from_it() {
        let args = scoped_source_args(
            Path::new("/repo"),
            Path::new("/repo/packages/rust"),
            "main",
            &[PathBuf::from("packages/node")],
            &[PathBuf::from("packages/rust/docs")],
        );
        assert!(args.contains(&":(top)packages/node".to_string()));
        assert!(args.contains(&":(top,exclude)packages/rust/docs".to_string()));
    }

    const NOWHERE: &str = "/nonexistent-tc-e2e";

    #[test]
    fn run_shell_reports_a_spawn_failure_with_the_command() {
        let err = run_shell(Path::new(NOWHERE), "true").unwrap_err();
        assert!(format!("{err:#}").contains("running e2e command `true`"));
    }

    #[test]
    fn pathspec_check_reports_a_spawn_failure() {
        let err = pathspec_matches_tracked(Path::new(NOWHERE), "src").unwrap_err();
        assert!(format!("{err:#}").contains("git ls-files -- src"));
    }

    #[test]
    fn diff_check_reports_a_spawn_failure() {
        let err = git_diff_changed(Path::new(NOWHERE), &["diff", "--quiet"]).unwrap_err();
        assert!(format!("{err:#}").contains("running `git diff --quiet`"));
    }

    #[test]
    fn capture_reports_a_spawn_failure() {
        let err = git_capture(Path::new(NOWHERE), &["rev-parse", "HEAD"]).unwrap_err();
        assert!(format!("{err:#}").contains("running `git rev-parse HEAD`"));
    }

    #[test]
    fn run_reports_a_spawn_failure() {
        let err = git_run(Path::new(NOWHERE), &["add", "-A"]).unwrap_err();
        assert!(format!("{err:#}").contains("running `git add -A`"));
    }

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
