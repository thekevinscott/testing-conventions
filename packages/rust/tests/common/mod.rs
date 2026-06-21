//! Shared test helper for the TypeScript mutation suites (#202).
//!
//! Stryker writes its report and sandbox *into the project dir*, so two runs in the
//! same fixture would collide when cargo runs tests in parallel. [`Staged`] copies a
//! fixture project into a unique temp dir — with `node_modules` symlinked to the shared
//! toolchain rather than copied — so every test gets a pristine, isolated project and
//! the committed fixtures are never written to.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway copy of a fixture project under `tests/fixtures/unit_mutation/typescript`,
/// removed on drop.
pub struct Staged(PathBuf);

impl Staged {
    /// Stage the named fixture project (`killed` / `survivors`) into a fresh temp dir.
    pub fn new(project: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unit_mutation/typescript");
        let dst = std::env::temp_dir().join(format!(
            "tc-mut-ts-{}-{}-{}",
            project,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&dst).expect("create staged project dir");
        for file in ["index.ts", "index.test.ts", "stryker.conf.json"] {
            std::fs::copy(fixtures.join(project).join(file), dst.join(file))
                .unwrap_or_else(|e| panic!("copy fixture {project}/{file}: {e}"));
        }
        // Symlink (not copy) the shared install — concurrent read-only resolution is safe.
        std::os::unix::fs::symlink(fixtures.join("node_modules"), dst.join("node_modules"))
            .expect("symlink node_modules");
        Staged(dst)
    }

    /// The staged project's root.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        // Remove the symlink first so we never recurse into the shared toolchain.
        let _ = std::fs::remove_file(self.0.join("node_modules"));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
