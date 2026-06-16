//! `e2e attest` / `e2e verify` (#17) — the e2e attestation nudge.
//!
//! `attest` runs the e2e suite locally and records that it ran against the
//! current commit; `verify` (a later slice, #68) confirms in CI that the latest
//! code commit is attested. The point is to *nudge* agents to run e2e locally —
//! CI never runs e2e, it only checks the committed attestation.
//!
//! This module implements both `attest` (#67) and `verify` (#68).

use std::path::Path;
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
    git_run(
        repo,
        &[
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            message.as_str(),
        ],
    )?;

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
/// Stub: the real check lands in the implementation commit; this compiles so the
/// red integration + e2e tests run against it.
pub fn verify(repo: &Path) -> Result<Verification> {
    let _ = repo;
    bail!("e2e verify is not implemented yet (#68)")
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
