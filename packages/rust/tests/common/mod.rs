//! Shared test helper for the mutation suites.
//!
//! The engines write into the project dir — Stryker and cosmic-ray both mutate files in
//! place (Stryker keeps its backup under `.stryker-tmp`) — so two runs in the same
//! fixture would collide when cargo runs tests in parallel, and the committed fixtures
//! would hold mutants while any run is live. [`Staged`] copies a fixture project into a unique temp dir (for
//! TypeScript, with the runner-only `node_modules` symlinked rather than copied) so every
//! test gets a pristine, isolated project and the committed fixtures are never written to.
//!
//! TypeScript also drives the bundled Node mutation adapter: the rule spawns
//! `packages/node/dist/mutation-cli.js`, whose path it receives explicitly. The integration
//! tests pass [`ts_adapter`] straight to [`testing_conventions::mutation::measure_typescript`];
//! the e2e tests pass it to the spawned binary as `--ts-mutation-adapter`.

// Each constructor is used by only some of the mutation test binaries.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::mutation::{Measurement, Survivor};

/// The `(count, survivors)` of a [`Measurement`] whose engine ran — panics on
/// [`Measurement::EngineNotRun`], failing the calling test.
pub fn expect_tested(measurement: Measurement) -> (usize, Vec<Survivor>) {
    match measurement {
        Measurement::Tested { count, survivors } => (count, survivors),
        Measurement::EngineNotRun => panic!("the engine must run for this measurement"),
    }
}

/// The line a diff-scoped `unit mutation` run prints when the changed lines hold nothing
/// mutatable — the engine-skipped pass, distinct from the all-killed success.
pub const ENGINE_NOT_RUN: &str = "unit mutation: no mutatable changed lines — engine not run";

/// The `<n>` from a passing run's counted success line — `unit mutation: no surviving
/// mutants — every mutation was caught (<n> mutant(s) tested)`. Panics (failing the
/// calling test) when stdout carries no such line or the line deviates from that exact
/// shape, so the assertion pins the full message format, not a substring.
pub fn tested_count(stdout: &str) -> u64 {
    const PREFIX: &str = "unit mutation: no surviving mutants — every mutation was caught (";
    const SUFFIX: &str = " mutant(s) tested)";
    let line = stdout
        .lines()
        .find(|line| line.starts_with("unit mutation: no surviving mutants"))
        .unwrap_or_else(|| panic!("no success line in stdout: {stdout:?}"));
    line.strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_suffix(SUFFIX))
        .and_then(|count| count.parse().ok())
        .unwrap_or_else(|| panic!("the success line does not state the tested count: {line:?}"))
}

/// The freshly-built TypeScript mutation adapter (`packages/node/dist/mutation/main.js`),
/// which the rule spawns for the TS arm. `CARGO_MANIFEST_DIR` is `packages/rust`,
/// so the sibling node package is one level up. Requires the node package to be built
/// (`npm run build` in `packages/node`, deps installed) — the Rust CI integration job does
/// both before the suite runs.
pub fn ts_adapter() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node/dist/mutation/main.js")
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

    /// Stage an upward-import TypeScript fixture (`upward_killed` / `upward_survivors`):
    /// the standard `{package.json, tsconfig.json, src/**, tests/**}` package layout whose
    /// `src/` imports `../package.json`. The package-level `tsconfig.json` is what a real
    /// consumer TS package carries, and its presence is what activates Stryker's ts-config
    /// machinery. The staged path is the **package root**; the mutation tests scan its
    /// `src/` subdirectory.
    pub fn upward(project: &str) -> Self {
        Self::stage(
            "typescript",
            project,
            &[
                "package.json",
                "tsconfig.json",
                "src/index.ts",
                "src/index.test.ts",
                "tests/integration/tiers.test.ts",
            ],
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
            let to = dst.join(file);
            if let Some(parent) = to.parent() {
                std::fs::create_dir_all(parent).expect("create staged subdirectory");
            }
            std::fs::copy(fixtures.join(project).join(file), to)
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

/// A throwaway git repo for the diff-scoped (`--base`) e2e runs, removed on drop. The
/// caller writes files, commits a baseline, and commits the change under test; the two
/// heads bound the `<base>...HEAD` diff the CLI is pointed at.
pub struct GitRepo(PathBuf);

impl GitRepo {
    pub fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-mut-e2e-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        Self::git(&root, &["init", "-q"]);
        Self::git(&root, &["config", "user.email", "test@example.com"]);
        Self::git(&root, &["config", "user.name", "Test"]);
        GitRepo(root)
    }

    pub fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    pub fn commit(&self, message: &str) {
        Self::git(&self.0, &["add", "-A"]);
        Self::git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    /// The current `HEAD` commit id — captured after the baseline commit to serve as `--base`.
    pub fn head(&self) -> String {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        assert!(out.status.success(), "git rev-parse failed");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

    /// The repo's root directory.
    pub fn path(&self) -> &Path {
        &self.0
    }

    fn git(dir: &Path, args: &[&str]) {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dir)
            .status()
            .expect("git should run");
        assert!(status.success(), "git {args:?} failed");
    }
}

impl Drop for GitRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// An isolated install of the **packed** npm package: `npm pack` over `packages/node`, the
/// tarball installed into a throwaway prefix. The resulting `node_modules` holds the
/// package's declared dependency closure and nothing from this repo's dev tree — the
/// topology `npx -y testing-conventions` runs in production, where a devDependency (e.g.
/// `typescript`) is absent from every resolution path. The repo's own suites run the
/// adapter from `packages/node`'s dev tree, where pnpm's hoisted devDependencies mask a
/// missing-declared-dependency bug; resolving the adapter from this install is what
/// surfaces it. Requires the node package built (`pnpm run build` in `packages/node`) and
/// registry access for the dependency install. Removed on drop.
pub struct PublishedInstall(PathBuf);

impl PublishedInstall {
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let node_package = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node");
        let dst = std::env::temp_dir().join(format!(
            "tc-published-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&dst).expect("create published install dir");
        let pack = std::process::Command::new("npm")
            .args(["pack", "--pack-destination"])
            .arg(&dst)
            .current_dir(&node_package)
            .output()
            .expect("npm pack should run");
        assert!(
            pack.status.success(),
            "npm pack failed: {}",
            String::from_utf8_lossy(&pack.stderr)
        );
        // `npm pack` prints the tarball filename it wrote as its last stdout line.
        let stdout = String::from_utf8_lossy(&pack.stdout);
        let tarball = stdout
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .expect("npm pack should print the tarball name")
            .trim()
            .to_string();
        std::fs::write(dst.join("package.json"), "{ \"private\": true }\n")
            .expect("write install prefix manifest");
        let install = std::process::Command::new("npm")
            .args(["install", "--no-audit", "--no-fund"])
            .arg(dst.join(&tarball))
            .current_dir(&dst)
            .output()
            .expect("npm install should run");
        assert!(
            install.status.success(),
            "npm install of the packed tarball failed: {}",
            String::from_utf8_lossy(&install.stderr)
        );
        PublishedInstall(dst)
    }

    /// The installed package's TypeScript mutation adapter — the executable the npm
    /// launcher hands the binary as `--ts-mutation-adapter` in production.
    pub fn adapter(&self) -> PathBuf {
        self.0
            .join("node_modules/testing-conventions/dist/mutation/main.js")
    }
}

impl Drop for PublishedInstall {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
