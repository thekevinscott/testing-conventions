//! Workflow guard — keep the reusable workflow in step with the CLI.
//!
//! The reusable workflow (`.github/workflows/testing-conventions.yml`) is the
//! documented `@v0` consumption path: a consumer pins `@v0`, and the workflow runs
//! the *published* `testing-conventions` binary via `npx`. When a CLI subcommand is
//! renamed or removed — e.g. `unit location` → `unit colocated-test` — but a
//! workflow still invokes the old name, every `@v0` consumer breaks with
//! `unrecognized subcommand`, silently: the workflow file is frozen at the tag
//! while `npx` keeps pulling the latest binary.
//!
//! This module is the deterministic guard against that drift. [`invocations`]
//! extracts every `testing-conventions …` call from a workflow file's shell, and
//! [`unknown_subcommands`] checks each one's subcommand chain against the binary's
//! own command tree (the source of truth, [`crate::command`]), flagging any chain
//! the binary no longer exposes. Run in CI against the reusable workflow it fails
//! the build the moment a workflow and the CLI fall out of step — before a release
//! can strand `@v0`.
//!
//! Extraction is a line-based, shell-aware scan, not a full GitHub Actions parser:
//! it tokenizes each non-comment line, finds the `testing-conventions` binary token
//! (the bare command word, optionally version-pinned `…@x` /
//! `…${VERSION:+@$VERSION}` — the `npx` / on-`PATH` form the reusable workflow and
//! the docs use), and reads the tokens after it as the invocation. That is the
//! deterministic bright-line; a path-qualified invocation (`./bin/testing-conventions`),
//! a subcommand split across a `\`-continuation, or one named in non-`run:` prose is
//! a documented limit.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::violation::Violation;

/// A single `testing-conventions` invocation found in a workflow file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    /// Workflow file the invocation was found in.
    pub file: PathBuf,
    /// 1-based line of the invocation.
    pub line: usize,
    /// Tokens after the `testing-conventions` binary name, in order — the
    /// subcommand chain first, then flags / values / positionals.
    pub args: Vec<String>,
}

/// Walk `path` — a workflow file, or a directory of them — and return every
/// `testing-conventions` invocation, in file-then-line order.
///
/// Directories are scanned recursively for `*.yml` / `*.yaml` files (sorted, for
/// deterministic output). Returns an error if a file or directory cannot be read.
pub fn invocations(path: impl AsRef<Path>) -> Result<Vec<Invocation>> {
    let path = path.as_ref();
    let mut files = Vec::new();
    collect_workflow_files(path, &mut files)?;
    files.sort();
    let mut out = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file)
            .with_context(|| format!("reading workflow `{}`", file.display()))?;
        for (i, line) in text.lines().enumerate() {
            if let Some(args) = line_invocation(line) {
                out.push(Invocation {
                    file: file.clone(),
                    line: i + 1,
                    args,
                });
            }
        }
    }
    Ok(out)
}

/// Collect workflow files under `path` into `out`: `path` itself when it is a
/// file, else every `*.yml` / `*.yaml` under it, recursively.
fn collect_workflow_files(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    let entries = std::fs::read_dir(path)
        .with_context(|| format!("reading directory `{}`", path.display()))?;
    for entry in entries {
        let entry =
            entry.with_context(|| format!("reading an entry under `{}`", path.display()))?;
        let child = entry.path();
        if child.is_dir() {
            collect_workflow_files(&child, out)?;
        } else if is_workflow_file(&child) {
            out.push(child);
        }
    }
    Ok(())
}

/// `true` when `path` has a `.yml` / `.yaml` extension (a GitHub Actions workflow).
fn is_workflow_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yml" | "yaml")
    )
}

/// The args of a `testing-conventions` invocation on `line`, or `None` if the
/// line has no such call. Comments are ignored and surrounding quotes stripped.
fn line_invocation(line: &str) -> Option<Vec<String>> {
    let tokens = tokenize(line);
    let pos = tokens.iter().position(|t| is_binary_token(t))?;
    Some(tokens[pos + 1..].to_vec())
}

/// `true` when `token` is the `testing-conventions` binary as a command word: bare,
/// or version-pinned (`testing-conventions@0.1.0`,
/// `testing-conventions${VERSION:+@$VERSION}`).
///
/// Only the bare command word is matched — the `npx` / on-`PATH` form the reusable
/// workflow and the "roll your own" docs use. A path-qualified token
/// (`packages/…/testing-conventions`, a `cp` / `install` argument) is deliberately
/// *not* matched, so a path that merely ends in the binary name isn't read as an
/// invocation.
fn is_binary_token(token: &str) -> bool {
    // Strip any version pin / shell expansion suffix, then require an exact match.
    let end = [token.find('@'), token.find("${")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(token.len());
    &token[..end] == "testing-conventions"
}

/// Split `line` into shell-ish tokens: whitespace separates, `'…'` and `"…"`
/// group (and are stripped), and an unquoted `#` starting a token begins a comment
/// that runs to end of line.
fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut cur = String::new();
    let mut started = false;
    let mut quote: Option<char> = None;
    for c in line.chars() {
        match quote {
            Some(q) => {
                if c == q {
                    quote = None;
                } else {
                    cur.push(c);
                }
            }
            None => match c {
                '#' if !started => break,
                '\'' | '"' => {
                    quote = Some(c);
                    started = true;
                }
                c if c.is_whitespace() => {
                    if started {
                        tokens.push(std::mem::take(&mut cur));
                        started = false;
                    }
                }
                c => {
                    cur.push(c);
                    started = true;
                }
            },
        }
    }
    if started {
        tokens.push(cur);
    }
    tokens
}

/// Of `invocations`, the ones whose subcommand chain names a subcommand the binary
/// — described by `root`, its clap command tree — no longer exposes.
///
/// Each invocation's leading tokens are walked against the tree: a token in a
/// subcommand position (the current command takes subcommands) must name one of
/// them, else it is flagged. The walk stops at the first flag (`-…`) — subcommands
/// precede options in clap — and at the first command that takes positionals rather
/// than subcommands, so a path argument is never mistaken for a subcommand.
pub fn unknown_subcommands(invocations: &[Invocation], root: &clap::Command) -> Vec<Violation> {
    let mut out = Vec::new();
    for inv in invocations {
        let mut node = root;
        for tok in &inv.args {
            // Flags begin the options/positionals section: the subcommand chain is
            // complete. A command that takes positionals (not subcommands) means
            // this token is an argument, not a subcommand to validate.
            if tok.starts_with('-') || !node.has_subcommands() {
                break;
            }
            match node.find_subcommand(tok.as_str()) {
                Some(sub) => node = sub,
                None => {
                    out.push(Violation {
                        file: inv.file.clone(),
                        line: inv.line,
                        rule: "no-unknown-subcommand",
                        message: format!(
                            "`{}` is not a `{}` subcommand — the published binary no longer exposes it",
                            tok,
                            node.get_name()
                        ),
                    });
                    break;
                }
            }
        }
    }
    out
}

/// Check `path` (a workflow file or directory): every `testing-conventions`
/// invocation must name a subcommand `root` still exposes. Returns one
/// [`Violation`] per offending invocation.
pub fn check(path: impl AsRef<Path>, root: &clap::Command) -> Result<Vec<Violation>> {
    Ok(unknown_subcommands(&invocations(path)?, root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    /// A throwaway directory tree, removed on drop.
    struct TempTree(PathBuf);

    impl TempTree {
        fn new(files: &[(&str, &str)]) -> Self {
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "tc-workflow-{}-{}",
                std::process::id(),
                COUNTER.fetch_add(1, Ordering::Relaxed),
            ));
            for (rel, content) in files {
                let path = root.join(rel);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(path, content).unwrap();
            }
            TempTree(root)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn tokenize_strips_quotes_and_groups() {
        assert_eq!(
            tokenize(r#"npx -y "testing-conventions${VERSION:+@$VERSION}" unit coverage"#),
            vec![
                "npx",
                "-y",
                "testing-conventions${VERSION:+@$VERSION}",
                "unit",
                "coverage",
            ]
        );
    }

    #[test]
    fn tokenize_stops_at_a_comment() {
        assert_eq!(
            tokenize("      # run testing-conventions later"),
            Vec::<String>::new()
        );
        assert_eq!(
            tokenize("testing-conventions check  # trailing note"),
            vec!["testing-conventions", "check"]
        );
    }

    #[test]
    fn is_binary_token_accepts_the_command_word() {
        assert!(is_binary_token("testing-conventions"));
        assert!(is_binary_token("testing-conventions@0.1.0"));
        assert!(is_binary_token("testing-conventions${VERSION:+@$VERSION}"));
    }

    #[test]
    fn is_binary_token_rejects_lookalikes() {
        assert!(!is_binary_token("testing-conventions.toml"));
        assert!(!is_binary_token("testing-conventions.yml@v0"));
        assert!(!is_binary_token("actions/checkout@v6"));
        assert!(!is_binary_token("npx"));
        // Path-qualified tokens — e.g. a `cp` / `install` argument — are not
        // invocations, even when they end in the binary name.
        assert!(!is_binary_token(
            "packages/rust/target/release/testing-conventions"
        ));
        assert!(!is_binary_token("$target/bin/testing-conventions"));
        assert!(!is_binary_token("./target/release/testing-conventions"));
    }

    #[test]
    fn line_invocation_reads_the_args_after_the_binary() {
        assert_eq!(
            line_invocation(
                "- run: npx -y testing-conventions unit location --language python src"
            ),
            Some(vec![
                "unit".to_string(),
                "location".to_string(),
                "--language".to_string(),
                "python".to_string(),
                "src".to_string(),
            ])
        );
        assert_eq!(line_invocation("- uses: actions/checkout@v6"), None);
    }

    #[test]
    fn invocations_scans_a_file_and_a_directory() {
        let tree = TempTree::new(&[
            ("ci.yml", "- run: testing-conventions check\n"),
            (
                "nested/more.yaml",
                "- run: testing-conventions unit lint --language rust .\n",
            ),
            ("notes.txt", "testing-conventions check\n"),
        ]);
        // Directory: both workflow files, not the .txt; sorted file-then-line.
        let dir = invocations(tree.path()).unwrap();
        assert_eq!(dir.len(), 2);
        assert_eq!(dir[0].args, vec!["check"]);
        assert_eq!(dir[0].line, 1);
        // Single file: just that file.
        let file = invocations(tree.path().join("ci.yml")).unwrap();
        assert_eq!(file.len(), 1);
    }

    #[test]
    fn invocations_errors_on_a_missing_path() {
        let missing = std::env::temp_dir().join("tc-workflow-does-not-exist-2b1c");
        assert!(invocations(&missing).is_err());
    }

    /// An [`Invocation`] from a bare token list (file/line are placeholders).
    fn inv(line: usize, args: &[&str]) -> Invocation {
        Invocation {
            file: PathBuf::from("ci.yml"),
            line,
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn unknown_subcommands_flags_a_renamed_nested_rule() {
        let v = unknown_subcommands(
            &[inv(9, &["unit", "location", "--language", "python", "src"])],
            &crate::command(),
        );
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].line, 9);
        assert_eq!(v[0].rule, "no-unknown-subcommand");
        // Named under its parent group, not the root.
        assert!(v[0].message.contains("`location`"), "{}", v[0].message);
        assert!(v[0].message.contains("`unit`"), "{}", v[0].message);
    }

    #[test]
    fn unknown_subcommands_flags_a_removed_top_level_command() {
        let v = unknown_subcommands(
            &[inv(1, &["unit-location", "--lang", "python", "src"])],
            &crate::command(),
        );
        assert_eq!(v.len(), 1);
        assert!(v[0].message.contains("`unit-location`"), "{}", v[0].message);
        assert!(
            v[0].message.contains("`testing-conventions`"),
            "{}",
            v[0].message
        );
    }

    #[test]
    fn unknown_subcommands_accepts_every_live_invocation() {
        let invs = [
            inv(
                1,
                &["unit", "colocated-test", "--language", "python", "src"],
            ),
            inv(2, &["unit", "coverage", "--language", "typescript", "src"]),
            inv(3, &["unit", "lint", "--language", "rust", "."]),
            inv(4, &["integration", "lint", "--language", "python", "src"]),
            // A leaf's positional must not be read as a subcommand.
            inv(5, &["packaging", "--language", "python", "dist"]),
            inv(6, &["check"]),
            // Flags-only and empty invocations have no subcommand to check.
            inv(7, &["--version"]),
            inv(8, &[]),
        ];
        assert!(unknown_subcommands(&invs, &crate::command()).is_empty());
    }
}
