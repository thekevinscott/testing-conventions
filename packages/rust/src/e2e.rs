//! `e2e attest` / `e2e verify` (#17) — the e2e attestation nudge.
//!
//! `attest` runs the e2e suite locally and records that it ran against the
//! current commit; `verify` (a later slice, #68) confirms in CI that the latest
//! code commit is attested. The point is to *nudge* agents to run e2e locally —
//! CI never runs e2e, it only checks the committed attestation.
//!
//! This is the stub surface for the red tests (#67): it compiles so the
//! integration + e2e tests run (and fail) against it; the behavior lands next.

use std::path::Path;

use anyhow::{bail, Result};
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
    let _ = (repo, command);
    bail!("e2e attest is not implemented yet (#67)")
}
