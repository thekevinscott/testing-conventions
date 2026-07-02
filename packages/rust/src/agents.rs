//! `install` (#232): write the testing contract into the repository's agent
//! context file (`AGENTS.md`) as a marker-delimited, hash-versioned block —
//! the beads (`bd init`) pattern. Idempotent: re-running refreshes the owned
//! region and touches nothing outside it.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{bail, Context};
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 1;
const BEGIN_OPEN: &str = "<!-- testing-conventions:begin ";
const END_MARKER: &str = "<!-- testing-conventions:end -->";

/// The managed region's content — the few non-negotiables plus pointers to
/// the canonical contract and the CLI. Thin on purpose: the consumer's file
/// is theirs; the full contract lives on the docs site.
const TEMPLATE: &str = "\
## Testing conventions

This repository enforces [testing-conventions](https://thekevinscott.github.io/testing-conventions/) in CI. The contract:

- Start every change with the docs update and red integration/e2e tests; CI witnesses them fail before the implementation lands.
- Colocate a unit test with every source file, and mock every collaborator in unit tests.
- Clear the coverage floor and kill the mutants on every line you touch.
- Ship each capability at parity across Python, TypeScript, and Rust.
- An exemption carries a written reason showing the isolation techniques you tried; near-zero is the bar.

Run the rules locally with the CLI: https://thekevinscott.github.io/testing-conventions/guide/cli
Machine-readable contract: https://thekevinscott.github.io/testing-conventions/llms.txt
";

/// The begin marker carries the schema version and the first 12 hex chars of
/// the SHA-256 of the region content, so staleness is visible at a glance.
fn begin_marker() -> String {
    let hex = Sha256::digest(TEMPLATE.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    format!("{BEGIN_OPEN}v{SCHEMA_VERSION} hash={} -->", &hex[..12])
}

/// Upsert the managed block into the file at `path`: create the file when
/// absent, append when the markers are missing, otherwise replace only the
/// region between the markers. A current block is a byte-identical no-op.
pub fn install(path: &Path) -> anyhow::Result<()> {
    if path
        .symlink_metadata()
        .map(|meta| meta.file_type().is_symlink())
        .unwrap_or(false)
    {
        bail!(
            "{} is a symlink; refusing to write through it",
            path.display()
        );
    }

    let existing = match fs::read_to_string(path) {
        Ok(text) => Some(text),
        Err(err) if err.kind() == ErrorKind::NotFound => None,
        Err(err) => return Err(err).with_context(|| format!("reading {}", path.display())),
    };

    let region = format!("{}\n{TEMPLATE}{END_MARKER}", begin_marker());
    let new = match &existing {
        None => format!("{region}\n"),
        Some(text) => {
            let bounds = text.find(BEGIN_OPEN).and_then(|start| {
                let end = text[start..].find(END_MARKER)?;
                Some((start, start + end + END_MARKER.len()))
            });
            match bounds {
                Some((start, end)) => format!("{}{region}{}", &text[..start], &text[end..]),
                None => {
                    let mut out = text.clone();
                    if !out.is_empty() && !out.ends_with('\n') {
                        out.push('\n');
                    }
                    if !out.is_empty() {
                        out.push('\n');
                    }
                    format!("{out}{region}\n")
                }
            }
        }
    };

    if existing.as_deref() == Some(new.as_str()) {
        return Ok(());
    }

    // Atomic write: temp file in the target's directory, then rename, so a
    // crash mid-write leaves the original intact.
    let name = path
        .file_name()
        .with_context(|| format!("{} has no file name", path.display()))?;
    let tmp = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join(format!(
            ".{}.tc-tmp-{}",
            name.to_string_lossy(),
            std::process::id()
        ));
    fs::write(&tmp, &new).with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("renaming {} over {}", tmp.display(), path.display()))
}
