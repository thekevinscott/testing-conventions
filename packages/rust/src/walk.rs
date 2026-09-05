use std::fs::DirEntry;
use std::path::Path;

use anyhow::{Context, Result};

/// The entry from a directory walk, with a read failure tagged by the directory it was under.
pub(crate) fn dir_entry(entry: std::io::Result<DirEntry>, dir: &Path) -> Result<DirEntry> {
    entry.with_context(|| format!("reading an entry under `{}`", dir.display()))
}

#[cfg(test)]
mod tests {
    use super::dir_entry;
    use std::path::Path;

    #[test]
    fn a_read_failure_names_the_directory_being_walked() {
        let result = dir_entry(Err(std::io::Error::other("boom")), Path::new("/some/dir"));
        let message = format!("{:#}", result.unwrap_err());
        assert!(message.contains("reading an entry under `/some/dir`"), "{message}");
        assert!(message.contains("boom"), "{message}");
    }

    #[test]
    fn a_readable_entry_passes_through() {
        let dir = std::env::temp_dir().join(format!("tc-walk-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("probe"), "").unwrap();
        let entry = std::fs::read_dir(&dir).unwrap().next().unwrap();
        let passed = dir_entry(entry, &dir).unwrap();
        assert_eq!(passed.file_name(), std::ffi::OsStr::new("probe"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
