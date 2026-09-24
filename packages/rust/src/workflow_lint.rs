//! `workflow-lint` — flag a program encoded in GitHub Actions YAML.
//!
//! A `run:` or `actions/github-script` body should be wiring: a few straight-line commands, or a
//! lone guard around an early exit. Iteration, multi-branch dispatch, and text-munging make it a
//! program, and a program in YAML is untested, un-runnable locally, and drifts in silence. It
//! belongs in a tested package in the repository's own language, invoked as a one-line `run:`.
//!
//! This is a pragmatic scanner, not a shell parser: it flags the high-signal markers of "this is a
//! program" and tolerates straight-line glue. It favors precision over recall — a borderline body
//! that slips through is still worth extracting, because an extracted script is testable.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Straight-line bodies longer than this must move out, branch-free or not: past a dozen commands
/// the step is a script whatever its control flow.
pub const MAX_GLUE_LINES: usize = 12;

/// Shell keywords that, at a command position, mean iteration or multi-branch dispatch.
const LOGIC_KEYWORDS: [(&str, &str); 5] = [
    ("for", "for loop"),
    ("while", "while loop"),
    ("until", "until loop"),
    ("select", "select loop"),
    ("case", "case dispatch"),
];

/// Commands whose whole purpose is rewriting text, which is a transformation worth testing.
const MUNGING_COMMANDS: [&str; 2] = ["awk", "sed"];

/// One step whose body encodes logic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub file: PathBuf,
    /// 1-based line of the body the reasons were found in.
    pub line: usize,
    /// The step's `name`, else its `uses`, else a placeholder.
    pub step: String,
    /// Which kind of body: `run` or `github-script`.
    pub kind: &'static str,
    /// Why the body is too complex to stay inline, in the order found.
    pub reasons: Vec<String>,
}

/// Why `body` is too complex to live inline. Empty means it reads as wiring.
pub fn flag_reasons(body: &str) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    for (keyword, name) in LOGIC_KEYWORDS {
        if body.lines().any(|line| starts_command(line, keyword)) {
            reasons.push(name.to_string());
        }
    }
    for command in MUNGING_COMMANDS {
        if body.lines().any(|line| runs_command(line, command)) {
            reasons.push(command.to_string());
        }
    }
    let count = significant_lines(body);
    if count > MAX_GLUE_LINES {
        reasons.push(format!("{count} lines (> {MAX_GLUE_LINES})"));
    }
    reasons
}

/// How many lines of `body` carry a command — blanks, comments, and the `set -…` prologue are
/// bookkeeping, not length.
fn significant_lines(body: &str) -> usize {
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with("set -"))
        .count()
}

/// `true` when `line`'s first word is `keyword`. Matching the whole first word is what keeps
/// `before_hook` off the `for` marker and `casements=3` off the `case` one.
fn starts_command(line: &str, keyword: &str) -> bool {
    line.split_whitespace().next() == Some(keyword)
}

/// `true` when `command` runs anywhere in `line` — at the start, or after a pipe, a separator, or a
/// substitution opener. `echo x | awk …` is as much a transformation as a bare `awk`.
fn runs_command(line: &str, command: &str) -> bool {
    command_segments(line).any(|segment| segment.split_whitespace().next() == Some(command))
}

/// `line` split at the shell positions a fresh command can start after: `|`, `;`, `&`, and the
/// `$(` / `` ` `` substitution openers. Splitting on the single characters covers `||` and `&&`
/// too, since the extra empty segment matches nothing.
fn command_segments(line: &str) -> impl Iterator<Item = &str> {
    line.split(['|', ';', '&', '(', ')', '`', '{', '}'])
}

/// Every logic-bearing step in one workflow or composite-action document.
///
/// A document that does not parse yields no findings rather than an error: `workflow-lint` judges
/// the shape of a body it can read, and unparseable YAML is [actionlint]'s to report, with far
/// better messages than this check could give.
///
/// [actionlint]: https://github.com/rhysd/actionlint
pub fn find_violations(yaml_text: &str, file: impl AsRef<Path>) -> Vec<Finding> {
    let file = file.as_ref();
    let root = match marked_yaml::parse_yaml(0, yaml_text) {
        Ok(marked_yaml::types::Node::Mapping(root)) => root,
        _ => return Vec::new(),
    };

    let mut findings = Vec::new();
    for step in steps(&root) {
        let label = step_label(step);
        for (kind, body, line) in bodies(step) {
            let reasons = flag_reasons(&body);
            if !reasons.is_empty() {
                findings.push(Finding {
                    file: file.to_path_buf(),
                    line,
                    step: label.clone(),
                    kind,
                    reasons,
                });
            }
        }
    }
    findings
}

/// Every step mapping in a document: each job's `steps` for a workflow, `runs.steps` for a
/// composite action.
fn steps(
    root: &marked_yaml::types::MarkedMappingNode,
) -> Vec<&marked_yaml::types::MarkedMappingNode> {
    let mut out = Vec::new();
    if let Some(jobs) = root.get_mapping("jobs") {
        for (_, job) in jobs.iter() {
            if let Some(job) = job.as_mapping() {
                push_steps(job, &mut out);
            }
        }
    }
    if let Some(runs) = root.get_mapping("runs") {
        push_steps(runs, &mut out);
    }
    out
}

/// Append `owner`'s `steps` sequence, when it has one, to `out`.
fn push_steps<'a>(
    owner: &'a marked_yaml::types::MarkedMappingNode,
    out: &mut Vec<&'a marked_yaml::types::MarkedMappingNode>,
) {
    let Some(steps) = owner.get_sequence("steps") else {
        return;
    };
    out.extend(steps.iter().filter_map(|step| step.as_mapping()));
}

/// How the step names itself in a report: its `name`, else the action it `uses`.
fn step_label(step: &marked_yaml::types::MarkedMappingNode) -> String {
    step.get_scalar("name")
        .or_else(|| step.get_scalar("uses"))
        .map(|node| node.as_str().to_string())
        .unwrap_or_else(|| "<unnamed step>".to_string())
}

/// Each `(kind, body, line)` a step carries: its `run`, and the `script` of a `github-script` step.
fn bodies(step: &marked_yaml::types::MarkedMappingNode) -> Vec<(&'static str, String, usize)> {
    let mut out = Vec::new();
    if let Some(run) = step.get_scalar("run") {
        out.push(("run", run.as_str().to_string(), line_of(run.span())));
    }
    let uses = step
        .get_scalar("uses")
        .map(|node| node.as_str().to_string());
    if uses.is_some_and(|uses| uses.starts_with("actions/github-script")) {
        if let Some(script) = step
            .get_mapping("with")
            .and_then(|w| w.get_scalar("script"))
        {
            out.push((
                "github-script",
                script.as_str().to_string(),
                line_of(script.span()),
            ));
        }
    }
    out
}

/// The 1-based line a node starts on, or 0 when the parser recorded no position.
fn line_of(span: &marked_yaml::Span) -> usize {
    span.start().map(|marker| marker.line()).unwrap_or(0)
}

/// Every logic-bearing step under `path` — a workflow file, or a directory to search — in
/// file-then-line order.
///
/// A `path` that is not there yields no findings: a repository with no CI has nothing to judge, so
/// the default `.github` must not be an error.
pub fn scan(path: impl AsRef<Path>) -> Result<Vec<Finding>> {
    let path = path.as_ref();
    let mut files = Vec::new();
    if path.exists() {
        collect_ci_files(path, &mut files)?;
    }
    files.sort();

    let mut findings = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file)
            .with_context(|| format!("reading workflow `{}`", file.display()))?;
        findings.extend(find_violations(&text, &file));
    }
    Ok(findings)
}

/// Collect the YAML GitHub actually executes under `path` into `out`: `path` itself when it is a
/// file, else every qualifying file beneath it, recursively. A named file is always scanned, so a
/// caller can point the check anywhere.
fn collect_ci_files(path: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if path.is_file() {
        out.push(path.to_path_buf());
        return Ok(());
    }
    let entries = std::fs::read_dir(path)
        .with_context(|| format!("reading directory `{}`", path.display()))?;
    for entry in entries {
        let child = crate::walk::dir_entry(entry, path)?.path();
        if child.is_dir() {
            collect_ci_files(&child, out)?;
        } else if is_ci_file(&child) {
            out.push(child);
        }
    }
    Ok(())
}

/// `true` when GitHub reads `path` as a workflow or a composite action: a YAML file directly inside
/// a `workflows` directory, or one named `action.yml` / `action.yaml` anywhere.
///
/// Discovery mirrors GitHub's own rules rather than taking every `*.yml` in the tree, so a fixture
/// or a lockfile that happens to sit under `.github` is never mistaken for CI.
fn is_ci_file(path: &Path) -> bool {
    if !matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yml" | "yaml")
    ) {
        return false;
    }
    let name = path.file_name().and_then(|n| n.to_str());
    if matches!(name, Some("action.yml" | "action.yaml")) {
        return true;
    }
    path.parent()
        .and_then(Path::file_name)
        .and_then(|n| n.to_str())
        == Some("workflows")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_line_glue_is_not_logic() {
        assert_eq!(
            flag_reasons(
                "pkg=@x/linux\nmkdir -p \"$pkg/bin\"\ninstall -m 755 bin/x \"$pkg/bin/x\""
            ),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_lone_guard_around_an_exit_is_glue() {
        let body = "set -euo pipefail\n\
                    private=$(gh api repos/x --jq .private)\n\
                    if [ \"$private\" = \"true\" ]; then\n\
                      echo \"::error::private\"\n\
                      exit 1\n\
                    fi\n\
                    echo ok\n";
        assert_eq!(flag_reasons(body), Vec::<String>::new());
    }

    #[test]
    fn each_loop_and_dispatch_keyword_is_named() {
        assert_eq!(
            flag_reasons("for name in $NAMES; do\n  npm publish \"$name\"\ndone"),
            vec!["for loop"]
        );
        assert_eq!(
            flag_reasons("while read -r l; do\n  echo $l\ndone"),
            vec!["while loop"]
        );
        assert_eq!(
            flag_reasons("until ok; do\n  sleep 1\ndone"),
            vec!["until loop"]
        );
        assert_eq!(
            flag_reasons("select x in a b; do\n  echo $x\ndone"),
            vec!["select loop"]
        );
        assert_eq!(
            flag_reasons("case \"$1\" in\n  a) echo a ;;\nesac"),
            vec!["case dispatch"]
        );
    }

    #[test]
    fn a_keyword_is_only_a_marker_at_a_command_position() {
        // `before` / `format` / `casements` share a prefix with a keyword and must not trip it.
        assert_eq!(
            flag_reasons("before_hook\nformat_output\ncasements=3"),
            Vec::<String>::new()
        );
        // A keyword as an argument is not a command either.
        assert_eq!(flag_reasons("echo for while case"), Vec::<String>::new());
    }

    #[test]
    fn text_munging_counts_wherever_the_command_runs() {
        assert_eq!(flag_reasons("awk -F/ '{print $2}' f"), vec!["awk"]);
        assert_eq!(flag_reasons("echo x | awk -F/ '{print $2}'"), vec!["awk"]);
        assert_eq!(flag_reasons("cat f | sed 's/a/b/'"), vec!["sed"]);
        assert_eq!(flag_reasons("v=$(sed -n 1p f)"), vec!["sed"]);
        assert_eq!(flag_reasons("a; sed -i s/x/y/ f"), vec!["sed"]);
        assert_eq!(flag_reasons("ok && awk 'END{print NR}' f"), vec!["awk"]);
    }

    #[test]
    fn a_munging_command_named_as_an_argument_is_not_running() {
        assert_eq!(
            flag_reasons("echo 'use awk for this'"),
            Vec::<String>::new()
        );
        assert_eq!(flag_reasons("apt-get install -y sed"), Vec::<String>::new());
    }

    #[test]
    fn a_long_straight_line_body_is_a_script_by_length() {
        let body = (0..MAX_GLUE_LINES + 3)
            .map(|i| format!("cmd{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(flag_reasons(&body), vec!["15 lines (> 12)"]);
    }

    #[test]
    fn the_length_count_ignores_bookkeeping() {
        // Exactly at the ceiling passes, and the prologue, comments, and blanks do not count.
        let body = (0..MAX_GLUE_LINES)
            .map(|i| format!("cmd{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(flag_reasons(&body), Vec::<String>::new());
        assert_eq!(
            flag_reasons("set -euo pipefail\n# a note\n\necho hi"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn every_reason_a_body_earns_is_reported() {
        let body = "for f in *; do\n  case $f in\n    a) sed -i s/x/y/ $f ;;\n  esac\ndone";
        assert_eq!(flag_reasons(body), vec!["for loop", "case dispatch", "sed"]);
    }

    const LOOP_WORKFLOW: &str = "jobs:\n  publish:\n    steps:\n      - name: Publish stubs\n        run: |\n          for name in $NAMES; do\n            npm publish \"$name\"\n          done\n";

    #[test]
    fn a_run_step_reports_its_file_name_and_line() {
        let findings = find_violations(LOOP_WORKFLOW, "w.yml");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].step, "Publish stubs");
        assert_eq!(findings[0].kind, "run");
        assert_eq!(findings[0].reasons, vec!["for loop"]);
        assert_eq!(findings[0].file, PathBuf::from("w.yml"));
        assert_eq!(findings[0].line, 6, "the body starts on line 6");
    }

    #[test]
    fn a_github_script_body_is_judged_too() {
        let yaml = "jobs:\n  label:\n    steps:\n      - uses: actions/github-script@v7\n        with:\n          script: |\n            for (const i of items) {\n              core.info(i)\n            }\n";
        let findings = find_violations(yaml, "w.yml");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].kind, "github-script");
        assert_eq!(findings[0].step, "actions/github-script@v7");
        assert_eq!(findings[0].reasons, vec!["for loop"]);
    }

    #[test]
    fn another_actions_script_input_is_not_a_github_script_body() {
        let yaml = "jobs:\n  j:\n    steps:\n      - uses: some/other@v1\n        with:\n          script: |\n            for x in 1 2; do echo $x; done\n";
        assert_eq!(find_violations(yaml, "w.yml"), Vec::new());
    }

    #[test]
    fn a_composite_action_is_scanned_through_runs_steps() {
        let yaml = "runs:\n  using: composite\n  steps:\n    - name: Munge\n      shell: bash\n      run: cat f | sed 's/a/b/'\n";
        let findings = find_violations(yaml, "action.yml");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].step, "Munge");
        assert_eq!(findings[0].reasons, vec!["sed"]);
    }

    #[test]
    fn a_clean_workflow_yields_nothing() {
        let yaml = "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n      - run: pnpm install --frozen-lockfile\n      - name: Test\n        run: pnpm test\n";
        assert_eq!(find_violations(yaml, "w.yml"), Vec::new());
    }

    #[test]
    fn a_step_without_a_name_falls_back_to_its_uses_then_a_placeholder() {
        let named_by_uses = "jobs:\n  j:\n    steps:\n      - uses: actions/setup-node@v4\n        run: for x in 1; do echo $x; done\n";
        assert_eq!(
            find_violations(named_by_uses, "w.yml")[0].step,
            "actions/setup-node@v4"
        );

        let unnamed = "jobs:\n  j:\n    steps:\n      - run: for x in 1; do echo $x; done\n";
        assert_eq!(find_violations(unnamed, "w.yml")[0].step, "<unnamed step>");
    }

    #[test]
    fn unparseable_yaml_is_actionlints_to_report_not_ours() {
        assert_eq!(find_violations("jobs: [unclosed\n", "w.yml"), Vec::new());
    }

    #[test]
    fn a_document_with_no_steps_anywhere_yields_nothing() {
        assert_eq!(find_violations("name: CI\non: push\n", "w.yml"), Vec::new());
        assert_eq!(find_violations("- a\n- b\n", "w.yml"), Vec::new());
        assert_eq!(find_violations("", "w.yml"), Vec::new());
        // A job, or a composite `runs:`, that declares no steps at all.
        assert_eq!(
            find_violations("jobs:\n  a:\n    runs-on: ubuntu-latest\n", "w.yml"),
            Vec::new()
        );
        assert_eq!(
            find_violations("runs:\n  using: node20\n  main: index.js\n", "action.yml"),
            Vec::new()
        );
    }

    #[test]
    fn a_job_that_is_not_a_mapping_is_skipped() {
        // `jobs:` holding a scalar is nonsense GitHub would reject; it must not panic here.
        assert_eq!(find_violations("jobs:\n  a: 3\n", "w.yml"), Vec::new());
    }

    #[test]
    fn a_github_script_step_carrying_no_script_has_no_body() {
        let yaml = "jobs:\n  j:\n    steps:\n      - uses: actions/github-script@v7\n";
        assert_eq!(find_violations(yaml, "w.yml"), Vec::new());
    }

    #[test]
    fn discovery_follows_githubs_own_rules() {
        assert!(is_ci_file(Path::new(".github/workflows/ci.yml")));
        assert!(is_ci_file(Path::new(".github/workflows/ci.yaml")));
        assert!(is_ci_file(Path::new(".github/actions/detect/action.yml")));
        assert!(is_ci_file(Path::new("tools/thing/action.yaml")));

        // Not CI: a lockfile or fixture that merely sits under `.github`, a nested file GitHub
        // never reads as a workflow, and a non-YAML extension.
        assert!(!is_ci_file(Path::new(
            ".github/selftest/clean/pnpm-lock.yaml"
        )));
        assert!(!is_ci_file(Path::new(
            ".github/workflows/nested/deep/ci.yml"
        )));
        assert!(!is_ci_file(Path::new(".github/workflows/notes.md")));
    }

    #[test]
    fn an_unreadable_directory_names_itself() {
        let err = collect_ci_files(
            Path::new("tc-workflow-lint-no-such-directory"),
            &mut Vec::new(),
        )
        .unwrap_err();
        assert!(
            format!("{err:#}").contains("reading directory"),
            "got: {err:#}"
        );
    }

    #[test]
    fn every_job_in_a_workflow_is_scanned() {
        let yaml = "jobs:\n  a:\n    steps:\n      - run: for x in 1; do :; done\n  b:\n    steps:\n      - run: cat f | sed s/a/b/\n";
        let findings = find_violations(yaml, "w.yml");
        assert_eq!(findings.len(), 2);
    }
}
