//! The subcommand walk behind the workflow guard: an invocation's token chain against the
//! CLI's clap tree.

use crate::violation::Violation;
use crate::workflow::Invocation;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn inv(line: usize, args: &[&str]) -> Invocation {
        Invocation {
            file: std::path::PathBuf::from("ci.yml"),
            line,
            args: args.iter().map(|s| s.to_string()).collect(),
        }
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
        let m = &flagged[0].message;
        assert!(m.contains("location"), "{m}");
        assert!(
            unknown_subcommands(&[inv(2, &["--config", "x", "unit", "coverage"])], &root)
                .is_empty()
        );
    }

    #[test]
    fn a_flag_carrying_its_value_inline_consumes_no_extra_token() {
        let root = clap::Command::new("tc").arg(
            clap::Arg::new("config")
                .long("config")
                .action(clap::ArgAction::Set),
        );
        assert!(flag_takes_value(&root, "--config"));
        assert!(!flag_takes_value(&root, "--config=x"));
    }

    #[test]
    fn a_boolean_flag_consumes_no_value() {
        let root = clap::Command::new("tc").arg(
            clap::Arg::new("verbose")
                .long("verbose")
                .action(clap::ArgAction::SetTrue),
        );
        assert!(!flag_takes_value(&root, "--verbose"));
    }

    #[test]
    fn a_short_flag_consumes_its_value() {
        let root = clap::Command::new("tc").arg(
            clap::Arg::new("config")
                .short('c')
                .action(clap::ArgAction::Set),
        );
        assert!(flag_takes_value(&root, "-c"));
        assert!(!flag_takes_value(&root, "-x"));
    }

    #[test]
    fn an_invocation_stopping_at_a_parent_subcommand_ends_the_walk() {
        // `tc unit` names a real subcommand and nothing after it. The walk has to end on the
        // token list running out, not only on reaching a node with no children — clap itself
        // rejects the incomplete invocation, and this rule has nothing to say about it.
        let root = clap::Command::new("tc")
            .subcommand(clap::Command::new("unit").subcommand(clap::Command::new("coverage")));
        assert!(unknown_subcommands(&[inv(1, &["unit"])], &root).is_empty());
    }
}
