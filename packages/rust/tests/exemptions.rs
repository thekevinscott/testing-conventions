//! Integration tests for the exemption-approval gate (#229): `testing-conventions
//! exemptions --base <REF> [--approved]`.
//!
//! The gate's first half is a pure, deterministic command — the same shape as co-change /
//! changed-line coverage. It diffs the `[[<language>.exempt]]` entries between `<base>`
//! and the working tree's config and exits non-zero when the diff **adds or modifies**
//! one, so every new or broadened exemption costs a human greenlight. The greenlight is
//! `--approved` (the reusable workflow sets it only when a non-author reviewer applied the
//! `tc:exemption-approved` label). The gate keys on the **whole entry** (path + rules +
//! `lines` scope + reason), so removing or leaving an entry byte-for-byte unchanged is
//! free, while widening a line scope (#226), lifting an extra rule, or even rewording the
//! reason gates.
//!
//! Each test builds a throwaway git repo: a `base` commit carrying the "before" config,
//! then the "after" config in the working tree + a commit on top, so `<base>...HEAD` is
//! the change under test. These drive the CLI (`run`) end to end; `exemptions_e2e.rs`
//! covers the built binary as a subprocess.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::run;

/// A throwaway git repo, removed on drop. A test writes the "before" config, `commit`s
/// it, captures `head()` as the `base`, then writes the "after" config and commits.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-exemptions-{}-{}-{}",
            slug,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        std::fs::create_dir_all(&root).unwrap();
        git(&root, &["init", "-q"]);
        git(&root, &["config", "user.email", "test@example.com"]);
        git(&root, &["config", "user.name", "Test"]);
        TempRepo(root)
    }

    /// Write `contents` to `rel`, creating parent directories.
    fn write(&self, rel: &str, contents: &str) {
        let path = self.0.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    /// Stage everything and commit, advancing HEAD.
    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    /// The current HEAD SHA — captured as the `base` before writing the "after".
    fn head(&self) -> String {
        let out = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.0)
            .output()
            .expect("git rev-parse should run");
        assert!(out.status.success(), "git rev-parse failed");
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .expect("git should run");
    assert!(status.success(), "git {args:?} failed");
}

/// Result of `testing-conventions exemptions --base <base> --config <repo>/<config>`,
/// run in-process (without the `--approved` greenlight).
fn run_exemptions(repo: &TempRepo, base: &str, config: &str) -> anyhow::Result<i32> {
    run_exemptions_with(repo, base, config, false)
}

/// As [`run_exemptions`], but passes `--approved` when `approved` is set — the human
/// greenlight the reusable workflow supplies once a non-author reviewer applies the label.
fn run_exemptions_with(
    repo: &TempRepo,
    base: &str,
    config: &str,
    approved: bool,
) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "exemptions".into(),
        "--base".into(),
        base.into(),
        "--config".into(),
        repo.0.join(config).into_os_string(),
    ];
    if approved {
        argv.push("--approved".into());
    }
    run(argv)
}

const CONFIG: &str = "testing-conventions.toml";
const COVERAGE: &str = "[python]\ncoverage = { branch = true, fail_under = 100 }\n";

/// A whole-file `[[python.exempt]]` block for `path` lifting `rules` (no `lines`).
fn whole(path: &str, rules: &str, reason: &str) -> String {
    format!("\n[[python.exempt]]\npath = \"{path}\"\nrules = [{rules}]\nreason = \"{reason}\"\n")
}

/// A line-scoped `[[python.exempt]]` block (#226): a `coverage` entry over `lines`.
fn scoped(path: &str, lines: &str, reason: &str) -> String {
    format!(
        "\n[[python.exempt]]\npath = \"{path}\"\nrules = [\"coverage\"]\nlines = [{lines}]\nreason = \"{reason}\"\n"
    )
}

// ---- Adding an exemption gates --------------------------------------------

#[test]
fn a_newly_added_exemption_exits_nonzero() {
    // The defining red case: the base config has no exemption; the diff adds one.
    let repo = TempRepo::new("py-add");
    repo.write(CONFIG, COVERAGE);
    repo.commit("base: no exemptions");
    let base = repo.head();

    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            whole("mypkg/cli.py", "\"colocated-test\"", "shim")
        ),
    );
    repo.commit("add an exemption");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
}

#[test]
fn an_unchanged_exemption_exits_zero() {
    // Keeping an existing exemption is free — only additions/modifications gate.
    let repo = TempRepo::new("py-unchanged");
    let with_exempt = format!(
        "{COVERAGE}{}",
        whole("mypkg/cli.py", "\"colocated-test\"", "shim")
    );
    repo.write(CONFIG, &with_exempt);
    repo.commit("base: one exemption");
    let base = repo.head();

    // Touch something unrelated; the exempt table is identical.
    repo.write("README.md", "# hi\n");
    repo.commit("unrelated change");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 0);
}

#[test]
fn removing_an_exemption_exits_zero() {
    // Dropping an exemption is always allowed — it tightens, never loosens.
    let repo = TempRepo::new("py-remove");
    let with_exempt = format!(
        "{COVERAGE}{}",
        whole("mypkg/cli.py", "\"colocated-test\"", "shim")
    );
    repo.write(CONFIG, &with_exempt);
    repo.commit("base: one exemption");
    let base = repo.head();

    repo.write(CONFIG, COVERAGE);
    repo.commit("remove the exemption");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 0);
}

// ---- Modifying an exemption gates -----------------------------------------

#[test]
fn adding_a_rule_to_an_existing_entry_exits_nonzero() {
    // Lifting an *additional* rule changes the entry, so its whole-entry identity differs
    // from base — a modification, which gates. (Two whole-file rules, valid under #226.)
    let repo = TempRepo::new("py-add-rule");
    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            whole("mypkg/cli.py", "\"colocated-test\"", "shim")
        ),
    );
    repo.commit("base: lifts colocated-test only");
    let base = repo.head();

    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            whole("mypkg/cli.py", "\"colocated-test\", \"co-change\"", "shim")
        ),
    );
    repo.commit("also lift co-change");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
}

#[test]
fn modifying_an_existing_entry_exits_nonzero() {
    // Modifying an entry gates, even a reworded reason: the gate keys on the *whole*
    // entry, so an agent can't quietly broaden an exemption's scope or justification.
    let repo = TempRepo::new("py-reason");
    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            whole("mypkg/cli.py", "\"colocated-test\"", "old reason")
        ),
    );
    repo.commit("base");
    let base = repo.head();

    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            whole(
                "mypkg/cli.py",
                "\"colocated-test\"",
                "a more thorough reason"
            )
        ),
    );
    repo.commit("reword the reason");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
}

#[test]
fn widening_a_line_scope_exits_nonzero() {
    // The #226 hole you can't dodge: broadening a line-scoped `coverage` exemption is a
    // modification (the line set is part of the entry's identity), so it gates.
    let repo = TempRepo::new("py-widen");
    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            scoped("mypkg/cfg.py", "\"12-13\"", "dead branch")
        ),
    );
    repo.commit("base: exempt lines 12-13");
    let base = repo.head();

    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            scoped("mypkg/cfg.py", "\"12-200\"", "dead branch")
        ),
    );
    repo.commit("widen to lines 12-200");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
}

#[test]
fn an_equivalent_line_spec_exits_zero() {
    // Re-spelling the same lines (`[12, 13]` vs `["12-13"]`) is the same scope, not a
    // change — the line set is range-expanded before comparison.
    let repo = TempRepo::new("py-equiv-lines");
    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            scoped("mypkg/cfg.py", "12, 13", "dead branch")
        ),
    );
    repo.commit("base: lines [12, 13]");
    let base = repo.head();

    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            scoped("mypkg/cfg.py", "\"12-13\"", "dead branch")
        ),
    );
    repo.commit("rewrite as range 12-13");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 0);
}

// ---- The human greenlight -------------------------------------------------

#[test]
fn the_approved_greenlight_lets_an_added_or_changed_exemption_pass() {
    // With the human greenlight (--approved), a newly-added exemption passes; without it,
    // the same diff fails. This is the binary approve/not decision the label drives.
    let repo = TempRepo::new("py-approved");
    repo.write(CONFIG, COVERAGE);
    repo.commit("base: no exemptions");
    let base = repo.head();
    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            whole("mypkg/cli.py", "\"colocated-test\"", "shim")
        ),
    );
    repo.commit("add an exemption");

    // Red without approval…
    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
    // …green once a human greenlights it.
    assert_eq!(run_exemptions_with(&repo, &base, CONFIG, true).unwrap(), 0);
}

// ---- Anti-loophole & config presence --------------------------------------

#[test]
fn a_config_file_added_with_an_exemption_exits_nonzero() {
    // Anti-loophole: a config that didn't exist at base can't smuggle exemptions in — an
    // absent base config means *every* HEAD exemption is newly added.
    let repo = TempRepo::new("new-config");
    repo.write("src/widget.py", "x = 1\n");
    repo.commit("base: no config file");
    let base = repo.head();

    repo.write(
        CONFIG,
        &format!(
            "{COVERAGE}{}",
            whole("src/widget.py", "\"colocated-test\"", "generated")
        ),
    );
    repo.commit("add a config file with an exemption");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
}

#[test]
fn no_config_anywhere_exits_zero() {
    // Zero-config drop-in: no config at base or HEAD means no exemptions either way.
    let repo = TempRepo::new("no-config");
    repo.write("src/widget.py", "x = 1\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("src/widget.py", "x = 2\n");
    repo.commit("edit, still no config");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 0);
}

// ---- Cross-language parity (#229: one schema, language-agnostic) -----------

#[test]
fn a_new_typescript_exemption_exits_nonzero() {
    let repo = TempRepo::new("ts-add");
    repo.write(CONFIG, "[typescript]\n");
    repo.commit("base");
    let base = repo.head();
    repo.write(
        CONFIG,
        "[typescript]\n\n[[typescript.exempt]]\npath = \"index.ts\"\n\
         rules = [\"colocated-test\"]\nreason = \"re-export barrel\"\n",
    );
    repo.commit("add a ts exemption");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
}

#[test]
fn a_new_rust_exemption_exits_nonzero() {
    let repo = TempRepo::new("rust-add");
    repo.write(CONFIG, "[rust]\n");
    repo.commit("base");
    let base = repo.head();
    repo.write(
        CONFIG,
        "[rust]\n\n[[rust.exempt]]\npath = \"build.rs\"\n\
         rules = [\"coverage\"]\nlines = [\"1-3\"]\nreason = \"generated\"\n",
    );
    repo.commit("add a rust exemption");

    assert_eq!(run_exemptions(&repo, &base, CONFIG).unwrap(), 1);
}

// ---- Errors ----------------------------------------------------------------

#[test]
fn an_unresolvable_base_ref_is_an_error() {
    // A base that can't be resolved must surface, never silently pass as "clean".
    let repo = TempRepo::new("bad-base");
    repo.write(CONFIG, COVERAGE);
    repo.commit("base");

    assert!(run_exemptions(&repo, "no-such-ref", CONFIG).is_err());
}
