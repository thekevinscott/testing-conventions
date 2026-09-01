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

/// The line a run prints when the engine ran but produced no mutants to judge — the
/// zero-count pass, distinct from both the counted all-caught success and the
/// engine-skipped pass.
pub const NOTHING_TESTED: &str = "unit mutation: the engine found no mutants to test";

/// The `<n>` from a passing run's counted success line — `unit mutation: no surviving
/// mutants — every mutation was caught (<n> mutant(s) tested)`. Panics unless stdout
/// carries that exact line, so the assertion pins the full message format.
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
/// which the rule spawns for the TS arm.
pub fn ts_adapter() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../node/dist/mutation/main.js")
}

/// A throwaway copy of a fixture project under `tests/fixtures/unit_mutation/<lang>`,
/// removed on drop.
pub struct Staged(PathBuf);

impl Staged {
    /// Stage a TypeScript fixture (`killed` / `survivors`) in the default package layout. The
    /// package-level `tsconfig.json` activates Stryker's ts-config machinery.
    pub fn new(project: &str) -> Self {
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

    /// Stage a TypeScript fixture that carries a package-root `vitest.config.ts`
    /// (`config_include`), whose `include` is written relative to that root.
    pub fn configured(project: &str) -> Self {
        Self::stage(
            "typescript",
            project,
            &[
                "package.json",
                "tsconfig.json",
                "vitest.config.ts",
                "src/index.ts",
                "src/index.test.ts",
                "tests/integration/tiers.test.ts",
            ],
            true,
        )
    }

    /// Stage a loose TypeScript fixture (`loose_killed` / `loose_survivors`): the flat,
    /// no-manifest case, where the staged path is both package root and scan path.
    pub fn loose(project: &str) -> Self {
        Self::stage(
            "typescript",
            project,
            &["index.ts", "index.test.ts", "stryker.conf.json"],
            true,
        )
    }

    /// Stage a Python fixture (`killed` / `survivors`) in the default package layout.
    pub fn python(project: &str) -> Self {
        Self::stage(
            "python",
            project,
            &[
                "pyproject.toml",
                "src/calc.py",
                "src/calc_test.py",
                "tests/integration/tiers_test.py",
            ],
            false,
        )
    }

    /// Stage the Python fixture whose colocated suite is nested a directory below the scan
    /// path (`nested_tests`), the shape a package with submodules has.
    pub fn python_nested(project: &str) -> Self {
        Self::stage(
            "python",
            project,
            &[
                "pyproject.toml",
                "src/calc.py",
                "src/calc_test.py",
                "src/pkg/__init__.py",
                "src/pkg/deep.py",
                "src/pkg/deep_test.py",
                "tests/integration/tiers_test.py",
            ],
            false,
        )
    }

    /// Stage a loose Python fixture (`loose_killed` / `loose_survivors`): the flat,
    /// no-manifest case, where the staged path is both package root and scan path.
    pub fn python_loose(project: &str) -> Self {
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
            std::os::unix::fs::symlink(fixtures.join("node_modules"), dst.join("node_modules"))
                .expect("symlink node_modules");
        }
        Staged(dst)
    }

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

    pub fn head(&self) -> String {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        assert!(out.status.success(), "git rev-parse failed");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }

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

/// An isolated install of the packed npm package — the resolution topology `npx -y
/// testing-conventions` runs in production. See `docs/internals/rust/testing.md`.
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
