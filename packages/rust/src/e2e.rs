//! `e2e attest` / `e2e verify` (#17) — the e2e attestation nudge.
//!
//! `attest` runs the e2e suite locally and records that it ran against the
//! current commit; `verify` (a later slice, #68) confirms in CI that the latest
//! code commit is attested. The point is to *nudge* agents to run e2e locally —
//! CI never runs e2e, it only checks the committed attestation.
//!
//! This module implements both `attest` (#67) and `verify` (#68).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Where the committed attestation lives, relative to the repo root.
pub const ATTESTATION_PATH: &str = "e2e-attestation.json";

/// A record of one local e2e run — written to disk and committed by [`attest`].
///
/// `commit` is the SHA of the code commit the run was made against (HEAD at
/// attest time); [`verify`](crate::e2e) (#68) checks it against the latest code
/// commit. The rest is information for humans — nothing is gated on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attestation {
    /// The command that was run (e.g. `pnpm run e2e`).
    pub command: String,
    /// When it ran, as a Unix timestamp (seconds).
    pub ran_at: u64,
    /// The command's exit code — recorded, never gated on.
    pub exit_code: i32,
    /// The commit the run was made against (HEAD at attest time).
    pub commit: String,
}

/// Run `command` in `repo`, write an [`Attestation`] naming the current HEAD to
/// `repo`/[`ATTESTATION_PATH`], and commit it on top. Returns the attestation.
///
/// Writes regardless of the command's exit code — this forces a *run*, not a
/// *pass*.
pub fn attest(repo: &Path, command: &str) -> Result<Attestation> {
    let commit = git_capture(repo, &["rev-parse", "HEAD"])
        .context("resolving HEAD — `e2e attest` must run inside a git repo with a commit")?;

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
        commit: commit.clone(),
    };

    // Write the attestation, then commit just that file on top — it names the
    // code commit it was run against (a commit can't name its own SHA).
    let path = repo.join(ATTESTATION_PATH);
    let json = serde_json::to_string_pretty(&attestation).context("serializing the attestation")?;
    std::fs::write(&path, format!("{json}\n"))
        .with_context(|| format!("writing {}", path.display()))?;

    git_run(repo, &["add", ATTESTATION_PATH])?;
    let short = &commit[..commit.len().min(7)];
    let message = format!("e2e attestation for {short}");
    // A plain commit that inherits the repo's signing policy: a repo requiring
    // verified signatures gets a signed (mergeable) attestation, instead of the
    // unsigned commit a forced `commit.gpgsign=false` would leave behind (#128).
    git_run(repo, &["commit", "-q", "-m", message.as_str()])?;

    Ok(attestation)
}

/// The outcome of [`verify`] — whether the committed attestation names the latest
/// code commit, and if not, why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verification {
    /// The attestation names the latest code commit — the gate passes.
    Fresh,
    /// No attestation file is present — the gate fails.
    Missing,
    /// An attestation is present but names an older commit than the latest code
    /// commit (code changed since it was attested) — the gate fails.
    Stale {
        /// The commit the attestation names.
        attested: String,
        /// The latest code commit (newest one touching a non-attestation path).
        latest: String,
    },
}

/// Verify that the committed attestation names the latest code commit (#68) — the
/// CI side of the nudge. Reads only the committed attestation: never runs e2e,
/// never inspects the recorded exit code or output.
///
/// Equivalent to [`verify_scoped`] with `scope` set to `repo` — the freshness
/// walk covers everything under `repo`, same as before #294.
pub fn verify(repo: &Path) -> Result<Verification> {
    verify_scoped(repo, repo)
}

/// Verify the committed attestation at `repo`, scoping the "latest code commit"
/// walk to `scope` instead of all of `repo` (#294).
///
/// `repo` and `scope` serve different roles: `repo` is where
/// `e2e-attestation.json` lives (the package root — a manifest's own natural
/// home), while `scope` is what counts as "code" for freshness (the directory a
/// `path`-scoped call actually named, which can be narrower — a package root
/// commonly also holds `tests/`, docs, and config files that aren't the
/// attestable source). `scope` must be `repo` or a descendant of it.
///
/// Equivalent to [`verify_since`] with `base` set to `None` — the walk covers
/// all reachable history, same as before #319.
pub fn verify_scoped(repo: &Path, scope: &Path) -> Result<Verification> {
    verify_since(repo, scope, None)
}

/// Equivalent to [`verify_extra_scoped`] with no extra roots and no excludes —
/// the freshness walk covers only `scope`, same as before #333.
pub fn verify_since(repo: &Path, scope: &Path, base: Option<&str>) -> Result<Verification> {
    verify_extra_scoped(repo, scope, base, &[], &[])
}

/// Verify the committed attestation at `repo`, joining **extra freshness roots**
/// outside `scope` into the "latest code commit" walk (#333).
///
/// `extra_scopes` are **repo-root-relative** directories — a shared source tree
/// that is a sibling of the package (a native core bound into several language
/// bindings), which no `scope` at-or-below the attestation directory can reach.
/// Their commits count as code just like commits under `scope`, so a change to
/// the shared tree stales an attestation whose own subtree it doesn't touch.
/// `excludes` are repo-root-relative directories carved back out of that union —
/// a feature-gated subtree (a core `cli/` compiled out of the bindings) whose
/// changes must not stale them. Because the roots may lie outside the package
/// subtree — that's the point — the descendant constraint stays on `scope` only.
///
/// The freshness rule is unchanged: the attestation must name the newest in-range
/// commit touching the **union** of `scope` and `extra_scopes`, minus `excludes`.
/// `base` diff-scopes the walk exactly as in [`verify_since`]. Empty
/// `extra_scopes` and `excludes` are byte-identical to [`verify_since`].
pub fn verify_extra_scoped(
    repo: &Path,
    scope: &Path,
    base: Option<&str>,
    extra_scopes: &[PathBuf],
    excludes: &[PathBuf],
) -> Result<Verification> {
    let path = repo.join(ATTESTATION_PATH);
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return Ok(Verification::Missing);
    };
    let attestation: Attestation =
        serde_json::from_str(&contents).context("parsing the attestation")?;

    match latest_code_commit(repo, scope, base, extra_scopes, excludes)? {
        // With `--base`, an empty walk means this branch introduced no scoped
        // code commit — there is nothing to re-attest, so it's fresh by
        // construction (mirrors the changed-line coverage/mutation gates, which
        // pass on an empty diff). This is what keeps the gate squash-safe: a
        // stale-on-base attestation never reds a PR that didn't touch the source.
        // Without `--base` the walk covers all history and only comes back empty
        // for a scope with no commits at all, where an equality check against the
        // attested SHA (below) is the right answer — same as before this flag.
        None if base.is_some() => Ok(Verification::Fresh),
        latest => {
            let latest = latest.unwrap_or_default();
            if attestation.commit == latest {
                Ok(Verification::Fresh)
            } else {
                Ok(Verification::Stale {
                    attested: attestation.commit,
                    latest,
                })
            }
        }
    }
}

/// The newest commit that changed any path other than the attestation file,
/// under the union of `scope` and `extra_scopes` minus `excludes` — the "latest
/// code commit" the attestation must name to be fresh. Uses an `:(exclude)`
/// pathspec so the attestation's own commit never counts as code. `extra_scopes`
/// and `excludes` are repo-root-relative (`:(top)` pathspec magic), so they match
/// from the top of the tree regardless of `repo` being a subdirectory cwd — that
/// is how a sibling tree outside `scope` joins the walk (#333). When `base` is
/// `Some`, the walk is restricted to the commits this branch introduced
/// (`<base>..HEAD`); `None` when that range holds no such commit. Without `base`
/// the walk covers all reachable history.
fn latest_code_commit(
    repo: &Path,
    scope: &Path,
    base: Option<&str>,
    extra_scopes: &[PathBuf],
    excludes: &[PathBuf],
) -> Result<Option<String>> {
    let pathspec = relative_pathspec(repo, scope);
    let mut args: Vec<String> = vec!["log".into(), "-1".into(), "--format=%H".into()];
    // `<base>..HEAD` — commits reachable from HEAD but not `base`, i.e. the ones
    // this branch introduced. A commit that also landed on `base` (a squash of an
    // earlier PR) is reachable from `base`, so it's excluded here — that's the
    // whole point.
    if let Some(base) = base {
        args.push(format!("{base}..HEAD"));
    }
    args.push("--".into());
    // Positive pathspecs: the package's own `scope`, plus each extra root as a
    // repo-root-relative `:(top)` pathspec so it matches from the tree top rather
    // than relative to `repo` (the package cwd git runs in).
    args.push(pathspec);
    for extra in extra_scopes {
        args.push(format!(":(top){}", extra.display()));
    }
    // Negative pathspecs: the attestation itself never counts as code, and each
    // excluded (feature-gated) subtree of an extra root is carved back out.
    args.push(format!(":(exclude){ATTESTATION_PATH}"));
    for exclude in excludes {
        args.push(format!(":(top,exclude){}", exclude.display()));
    }
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = git_capture(repo, &arg_refs)?;
    Ok((!out.is_empty()).then_some(out))
}

/// `scope` as a pathspec relative to `repo` (git resolves pathspecs relative to
/// the invocation's cwd, which is always `repo` here). `.` when `scope` is
/// `repo` itself — byte-identical to the pre-#294 pathspec.
fn relative_pathspec(repo: &Path, scope: &Path) -> String {
    if scope == repo {
        return ".".to_string();
    }
    match scope.strip_prefix(repo) {
        Ok(rel) if !rel.as_os_str().is_empty() => rel.to_string_lossy().into_owned(),
        _ => scope.to_string_lossy().into_owned(),
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
