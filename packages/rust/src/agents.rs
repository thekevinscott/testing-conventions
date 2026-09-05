//! `install`: upsert the testing contract into the repository's agent context
//! file as a marker-delimited, hash-versioned block.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{anyhow, bail, Context};
use sha2::{Digest, Sha256};

const SCHEMA_VERSION: u32 = 1;
const BEGIN_OPEN: &str = "<!-- testing-conventions:begin ";
const END_MARKER: &str = "<!-- testing-conventions:end -->";

const TEMPLATE: &str = "\
## Testing conventions

This repository enforces [testing-conventions](https://thekevinscott.github.io/testing-conventions/) in CI. The contract:

- Start every change with the docs update and red integration/e2e tests; CI witnesses them fail before the implementation lands.
- Colocate a unit test with every source file, and mock every collaborator in unit tests.
- Clear the coverage floor and kill the mutants on every line you touch.
- Ship each capability at parity across Python, TypeScript, and Rust.
- An exemption carries a written reason showing the isolation techniques you tried; near-zero is the bar.

Machine-readable contract: https://thekevinscott.github.io/testing-conventions/llms.txt
";

/// The begin marker: the schema version and the first 12 hex chars of the region's SHA-256.
fn begin_marker() -> String {
    let hex = Sha256::digest(TEMPLATE.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    format!("{BEGIN_OPEN}v{SCHEMA_VERSION} hash={} -->", &hex[..12])
}

/// Upsert the managed block into the file at `path`: create when absent, append when no
/// marker is present, otherwise replace the region between the markers.
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
        Some(text) => match text.find(BEGIN_OPEN) {
            Some(start) => {
                let rel_end = text[start..].find(END_MARKER).ok_or_else(|| {
                    anyhow!(
                        "{}: a `testing-conventions` begin marker has no matching end marker \
                         — refusing to write, as replacing a partial block would delete \
                         surrounding content. Restore the `{END_MARKER}` marker (or remove the \
                         stray begin marker) and re-run.",
                        path.display()
                    )
                })?;
                let end = start + rel_end + END_MARKER.len();
                format!("{}{region}{}", &text[..start], &text[end..])
            }
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
        },
    };

    if existing.as_deref() == Some(new.as_str()) {
        return Ok(());
    }

    // Written to a temp file beside the target and renamed, so a crash mid-write leaves
    // the original intact.
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
    persist(&tmp, path, &new)
}

/// Write `contents` to `tmp`, then rename it over `path`.
fn persist(tmp: &Path, path: &Path, contents: &str) -> anyhow::Result<()> {
    fs::write(tmp, contents).with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(tmp, path)
        .with_context(|| format!("renaming {} over {}", tmp.display(), path.display()))
}

#[cfg(test)]
mod tests {
    use super::{install, persist};
    use std::path::Path;

    #[test]
    fn an_empty_path_reports_no_file_name() {
        let err = install(Path::new("")).unwrap_err();
        assert!(format!("{err:#}").contains("has no file name"));
    }

    #[test]
    fn a_failed_temp_write_names_the_temp_path() {
        let missing = Path::new("/nonexistent-tc/agents.tmp");
        let err = persist(missing, Path::new("/nonexistent-tc/AGENTS.md"), "x").unwrap_err();
        assert!(format!("{err:#}").contains("writing /nonexistent-tc/agents.tmp"));
    }

    #[test]
    fn a_failed_rename_names_both_paths() {
        let dir = std::env::temp_dir().join(format!("tc-agents-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let tmp = dir.join("block.tmp");
        let err = persist(&tmp, &dir.join("missing/AGENTS.md"), "x").unwrap_err();
        let message = format!("{err:#}");
        assert!(message.contains("renaming"), "{message}");
        assert!(message.contains("missing/AGENTS.md"), "{message}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
