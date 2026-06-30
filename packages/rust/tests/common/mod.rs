//! Shared test helper for the mutation suites (#202, #203).
//!
//! The engines write into the project dir — Stryker its sandbox, cosmic-ray mutates
//! files in place — so two runs in the same fixture would collide when cargo runs tests
//! in parallel. [`Staged`] copies a fixture project into a unique temp dir (for
//! TypeScript, with the runner-only `node_modules` symlinked rather than copied) so every
//! test gets a pristine, isolated project and the committed fixtures are never written to.
//!
//! TypeScript also drives the bundled Node mutation adapter (#246): the rule spawns
//! `packages/node/dist/mutation-cli.js`, found via the `TESTING_CONVENTIONS_TS_MUTATION_ADAPTER`
//! env var (the npm launcher sets it in production). The integration tests point it at the
//! freshly-built adapter via [`ensure_ts_adapter_env`]; the e2e tests pass the same path to
//! the spawned binary.

// Each constructor is used by only some of the mutation test binaries.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Once;

/// The freshly-built TypeScript mutation adapter (`packages/node/dist/mutation-cli.js`),
/// which the rule spawns for the TS arm (#246). `CARGO_MANIFEST_DIR` is `packages/rust`,
/// so the sibling node package is one level up. Requires the node package to be built
/// (`npm run build` in `packages/node`, deps installed) — the Rust CI integration job does
/// both before the suite runs.
pub fn ts_adapter() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node/dist/mutation-cli.js")
}

/// Point the rule at the built adapter by setting `TESTING_CONVENTIONS_TS_MUTATION_ADAPTER`
/// — the env var the npm launcher injects in production. The in-process integration tests
/// call this before [`testing_conventions::mutation::measure_typescript`]; the e2e tests
/// pass [`ts_adapter`] to the spawned binary instead. Set once, since every test sets the
/// same value; `std::env`'s internal lock makes the set/read race-free across the parallel
/// tests in this process.
pub fn ensure_ts_adapter_env() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        std::env::set_var("TESTING_CONVENTIONS_TS_MUTATION_ADAPTER", ts_adapter());
    });
}

/// A throwaway copy of a fixture project under `tests/fixtures/unit_mutation/<lang>`,
/// removed on drop.
pub struct Staged(PathBuf);

impl Staged {
    /// Stage a TypeScript fixture (`killed` / `survivors`), symlinking the runner-only
    /// `node_modules` (vitest) in — Stryker itself is bundled with and driven by the Node
    /// adapter, so the project supplies only its own test runner.
    pub fn new(project: &str) -> Self {
        Self::stage(
            "typescript",
            project,
            &["index.ts", "index.test.ts", "stryker.conf.json"],
            true,
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
