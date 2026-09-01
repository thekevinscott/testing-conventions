//! Workflow guard — flag a workflow invocation naming a subcommand the CLI no longer
//! exposes. Extraction is a line-based, shell-aware scan, not a GitHub Actions parser.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::violation::Violation;

/// A single `testing-conventions` invocation found in a workflow file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub file: PathBuf,
    /// 1-based line of the invocation.
    pub line: usize,
    /// Tokens after the `testing-conventions` binary name, in order.
    pub args: Vec<String>,
}

/// Every `testing-conventions` invocation under `path` — a workflow file or a directory
/// of them — in file-then-line order.
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

/// Collect workflow files under `path` into `out`: `path` itself when it is a file, else
/// every `*.yml` / `*.yaml` under it, recursively.
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

/// `true` when `path` has a `.yml` / `.yaml` extension.
fn is_workflow_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yml" | "yaml")
    )
}

/// The args of a `testing-conventions` invocation on `line`, or `None`. Comments are
/// ignored and surrounding quotes stripped.
fn line_invocation(line: &str) -> Option<Vec<String>> {
    let tokens = tokenize(line);
    let pos = tokens.iter().position(|t| is_binary_token(t))?;
    if !is_command_position(&tokens, pos) {
        return None;
    }
    Some(tokens[pos + 1..].to_vec())
}

/// `true` when the binary token at `pos` is the command being run, not an argument to
/// another command (`pip install testing-conventions` is an argument).
fn is_command_position(tokens: &[String], pos: usize) -> bool {
    let mut i = pos;
    // Stepping back over an `npx` launcher and its flags keeps `npx -y testing-conventions`
    // in command position.
    if i > 0 {
        let mut j = i;
        while j > 0 && tokens[j - 1].starts_with('-') {
            j -= 1;
        }
        if j > 0 && tokens[j - 1] == "npx" {
            i = j - 1;
        }
    }
    match i.checked_sub(1) {
        None => true,
        Some(prev) => is_command_boundary(&tokens[prev]),
    }
}

/// `true` when `token` ends a command: the YAML `run:` lead-in, or a shell separator.
fn is_command_boundary(token: &str) -> bool {
    matches!(token, "run:" | "&&" | "||" | "|" | ";" | "&" | "(" | "{")
}

/// `true` when `token` is the bare `testing-conventions` command word, optionally
/// version-pinned. A path-qualified token is not matched.
fn is_binary_token(token: &str) -> bool {
    // Strip any version pin / shell expansion suffix, then require an exact match.
    let end = [token.find('@'), token.find("${")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(token.len());
    &token[..end] == "testing-conventions"
}

/// Split `line` into shell-ish tokens: whitespace separates, `'…'` and `"…"` group (and
/// are stripped), and an unquoted `#` starting a token comments out the rest.
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

/// Of `invocations`, the ones whose subcommand chain names a subcommand the clap tree
/// `root` no longer exposes.
pub fn unknown_subcommands(invocations: &[Invocation], root: &clap::Command) -> Vec<Violation> {
    let mut out = Vec::new();
    for inv in invocations {
        let mut node = root;
        let mut i = 0;
        while i < inv.args.len() {
            // Past the last subcommand the remaining tokens are positionals, so walking on
            // would flag a path argument as an unknown subcommand.
            if !node.has_subcommands() {
                break;
            }
            let tok = &inv.args[i];
            if tok.starts_with('-') {
                i += if flag_takes_value(node, tok) { 2 } else { 1 };
                continue;
            }
            match node.find_subcommand(tok.as_str()) {
                Some(sub) => {
                    node = sub;
                    i += 1;
                }
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

/// `true` when the flag `token` is an option of `node` that consumes a following value,
/// so the subcommand walk must skip that value too.
fn flag_takes_value(node: &clap::Command, token: &str) -> bool {
    if token.contains('=') {
        return false;
    }
    let name = token.trim_start_matches('-');
    node.get_arguments().any(|arg| {
        let matches_long = arg.get_long() == Some(name);
        let matches_short = name.len() == 1 && arg.get_short().is_some_and(|c| name.starts_with(c));
        (matches_long || matches_short)
            && matches!(
                arg.get_action(),
                clap::ArgAction::Set | clap::ArgAction::Append
            )
    })
}

/// One [`Violation`] per invocation under `path` naming a subcommand `root` no longer
/// exposes.
pub fn check(path: impl AsRef<Path>, root: &clap::Command) -> Result<Vec<Violation>> {
    Ok(unknown_subcommands(&invocations(path)?, root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

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
            tokenize("testing-conventions install  # trailing note"),
            vec!["testing-conventions", "install"]
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
    fn line_invocation_ignores_a_package_install_line() {
        assert_eq!(
            line_invocation("- run: pip install testing-conventions pytest"),
            None
        );
        assert_eq!(
            line_invocation("- run: npm install -D testing-conventions"),
            None
        );
        assert_eq!(
            line_invocation("- run: cargo install testing-conventions"),
            None
        );
        assert_eq!(
            line_invocation("- run: testing-conventions install"),
            Some(vec!["install".to_string()])
        );
        assert_eq!(
            line_invocation("- run: npx -y testing-conventions install"),
            Some(vec!["install".to_string()])
        );
    }

    #[test]
    fn unknown_subcommands_validates_across_leading_global_flags() {
        let root = clap::Command::new("tc")
            .arg(
                clap::Arg::new("config")
                    .long("config")
                    .action(clap::ArgAction::Set),
            )
            .subcommand(clap::Command::new("unit").subcommand(clap::Command::new("coverage")));
        let flagged = unknown_subcommands(&[inv(1, &["--config", "x", "unit", "location"])], &root);
        assert_eq!(flagged.len(), 1, "{flagged:?}");
        assert!(
            flagged[0].message.contains("location"),
            "{}",
            flagged[0].message
        );
        assert!(
            unknown_subcommands(&[inv(2, &["--config", "x", "unit", "coverage"])], &root)
                .is_empty()
        );
    }

    #[test]
    fn invocations_scans_a_file_and_a_directory() {
        let tree = TempTree::new(&[
            ("ci.yml", "- run: testing-conventions install\n"),
            (
                "nested/more.yaml",
                "- run: testing-conventions unit lint --language rust .\n",
            ),
            ("notes.txt", "testing-conventions install\n"),
        ]);
        let dir = invocations(tree.path()).unwrap();
        assert_eq!(dir.len(), 2);
        assert_eq!(dir[0].args, vec!["install"]);
        assert_eq!(dir[0].line, 1);
        let file = invocations(tree.path().join("ci.yml")).unwrap();
        assert_eq!(file.len(), 1);
    }

    #[test]
    fn invocations_errors_on_a_missing_path() {
        let missing = std::env::temp_dir().join("tc-workflow-does-not-exist-2b1c");
        assert!(invocations(&missing).is_err());
    }

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
            inv(5, &["packaging", "--language", "python", "dist"]),
            inv(6, &["install"]),
            inv(7, &["--version"]),
            inv(8, &[]),
        ];
        assert!(unknown_subcommands(&invs, &crate::command()).is_empty());
    }
}
