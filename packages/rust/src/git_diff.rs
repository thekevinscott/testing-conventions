//! The `git diff --name-status` read behind the `co-change` rule.

use std::path::Path;
use std::process::Command;

use anyhow::{bail, Context, Result};

/// The diff status of a changed file, narrowed to what the rule acts on.
pub(crate) enum Status {
    /// `M` — content changed; a subject if the file still declares behavior.
    Modified,
    /// `D` — removed; a subject only if the source had a colocated test in base.
    Deleted,
    /// `A` (added) and the rest (`T`, …) — not a co-change subject.
    Other,
}

impl Status {
    /// The status from a `git diff --name-status` field; `--no-renames` makes it one letter.
    fn from_code(code: &str) -> Status {
        match code.chars().next() {
            Some('M') => Status::Modified,
            Some('D') => Status::Deleted,
            _ => Status::Other,
        }
    }
}

/// The status + `repo`-relative path of every file changed in `<base>...HEAD`, via
/// `git diff --name-status`. `--no-renames` shows a rename as a delete + an add, so the deleted
/// source is still held to its test; `--relative` scopes the diff to `repo`.
pub(crate) fn changed_entries(repo: &Path, base: &str) -> Result<Vec<(Status, String)>> {
    let range = format!("{base}...HEAD");
    // `core.quotepath=off` emits a non-ASCII path raw rather than octal-escaped, so a modified
    // `src/föö.py` reads back as a real file; `--no-ext-diff` blocks a configured external differ.
    let output = Command::new("git")
        .current_dir(repo)
        .args([
            "-c",
            "core.quotepath=off",
            "diff",
            "--name-status",
            "--no-ext-diff",
            "--no-renames",
            "--relative",
            &range,
        ])
        .output()
        .with_context(|| format!("running `git diff` in `{}`", repo.display()))?;
    if !output.status.success() {
        bail!(
            "`git diff {range}` failed in `{}`: {}",
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut entries = Vec::new();
    for line in stdout.lines() {
        if let Some((status, path)) = line.split_once('\t') {
            // A name with a `"`, a backslash, or a control byte still comes C-quoted even
            // with `core.quotepath=off`.
            let path = crate::git_path::unquote_c_path(path.trim_end_matches('\r'));
            let path = path.replace('\\', "/");
            entries.push((Status::from_code(status), path));
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_code_narrows_to_what_the_rule_acts_on() {
        assert!(matches!(Status::from_code("M"), Status::Modified));
        assert!(matches!(Status::from_code("D"), Status::Deleted));
        assert!(matches!(Status::from_code("A"), Status::Other));
        assert!(matches!(Status::from_code(""), Status::Other));
    }

    #[test]
    fn changed_entries_reports_a_spawn_failure() {
        let err = changed_entries(Path::new("/nonexistent-tc-git-diff"), "main")
            .err()
            .expect("the missing repo errors");
        assert!(format!("{err:#}").contains("running `git diff`"));
    }
}
