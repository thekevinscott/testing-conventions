//! Shared test helper for the mutation suites (#202, #203).
//!
//! The engines write into the project dir — Stryker its report/sandbox, cosmic-ray
//! mutates files in place — so two runs in the same fixture would collide when cargo
//! runs tests in parallel. [`Staged`] copies a fixture project into a unique temp dir
//! (for TypeScript, with `node_modules` symlinked to the shared toolchain rather than
//! copied) so every test gets a pristine, isolated project and the committed fixtures
//! are never written to.

// Each constructor is used by only some of the mutation test binaries.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// A throwaway copy of a fixture project under `tests/fixtures/unit_mutation/<lang>`,
/// removed on drop.
pub struct Staged(PathBuf);

impl Staged {
    /// Stage a TypeScript fixture (`killed` / `survivors`), symlinking the shared
    /// Stryker toolchain's `node_modules` in.
    pub fn new(project: &str) -> Self {
        Self::stage(
            "typescript",
            project,
            &["index.ts", "index.test.ts", "stryker.conf.json"],
            true,
        )
    }

    /// Stage a TypeScript fixture *without* the Stryker toolchain — no `node_modules`
    /// symlink. Exercises the not-installed path: the rule must fail clean via
    /// `npx --no-install` rather than silently download the deprecated `stryker` package.
    pub fn typescript_without_toolchain(project: &str) -> Self {
        Self::stage(
            "typescript",
            project,
            &["index.ts", "index.test.ts", "stryker.conf.json"],
            false,
        )
    }

    /// Stage a Python fixture (`killed` / `survivors`); cosmic-ray and pytest resolve
    /// from the ambient install, so there's no `node_modules` to link.
    pub fn python(project: &str) -> Self {
        Self::stage("python", project, &["calc.py", "calc_test.py"], false)
    }

    fn stage(lang: &str, project: &str, files: &[&str], link_node_modules: bool) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/unit_mutation")
            .join(lang);
        let dst = std::env::temp_dir().join(format!(
            "tc-mut-{}-{}-{}-{}",
            lang,
            project,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&dst).expect("create staged project dir");
        for file in files {
            std::fs::copy(fixtures.join(project).join(file), dst.join(file))
                .unwrap_or_else(|e| panic!("copy fixture {lang}/{project}/{file}: {e}"));
        }
        if link_node_modules {
            // Symlink (not copy) the shared install — concurrent read-only resolution is safe.
            std::os::unix::fs::symlink(fixtures.join("node_modules"), dst.join("node_modules"))
                .expect("symlink node_modules");
        }
        Staged(dst)
    }

    /// The staged project's root.
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Staged {
    fn drop(&mut self) {
        // Remove the node_modules symlink first (if any) so we never recurse into the
        // shared toolchain.
        let _ = std::fs::remove_file(self.0.join("node_modules"));
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
