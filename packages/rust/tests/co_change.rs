use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use testing_conventions::co_change::stale_sources;
use testing_conventions::colocated_test::Language;
use testing_conventions::run;

/// A throwaway git repo, removed on drop. Starts with no commits; a test writes
/// a baseline, `commit`s it, captures `head()` as the `base`, then mutates and
/// commits the "after" so `<base>...HEAD` is the change under test.
struct TempRepo(PathBuf);

impl TempRepo {
    fn new(slug: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "tc-co-change-{}-{}-{}",
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

    /// Delete `rel` from the working tree.
    fn remove(&self, rel: &str) {
        std::fs::remove_file(self.0.join(rel)).unwrap();
    }

    /// Stage everything and commit, advancing HEAD.
    fn commit(&self, message: &str) {
        git(&self.0, &["add", "-A"]);
        git(
            &self.0,
            &["-c", "commit.gpgsign=false", "commit", "-q", "-m", message],
        );
    }

    /// The current HEAD SHA — captured as the `base` before mutating.
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

/// The stale sources reported for `<base>...HEAD` (no exemptions), as `/`-joined
/// relative paths.
fn stale(repo: &TempRepo, base: &str, language: Language) -> Vec<String> {
    stale_sources(&repo.0, base, language, &BTreeSet::new())
        .expect("diffing a readable repo should succeed")
        .iter()
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .collect()
}

/// Result of `unit colocated-test <repo> --language <lang> --base <base> [--config
/// <repo>/<config>]`, run in-process. The commit-scoped co-change check
/// rides on `colocated-test`'s opt-in `--base` flag (presence + co-change), so
/// these cases drive that command.
fn run_co_change(
    repo: &TempRepo,
    language: &str,
    base: &str,
    config: Option<&str>,
) -> anyhow::Result<i32> {
    let mut argv: Vec<OsString> = vec![
        "testing-conventions".into(),
        "unit".into(),
        "colocated-test".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        language.into(),
        "--base".into(),
        base.into(),
    ];
    if let Some(name) = config {
        argv.push("--config".into());
        argv.push(repo.0.join(name).into_os_string());
    }
    run(argv)
}

/// Result of `unit colocated-test <repo> --language <lang>` with **no** `--base`:
/// the presence-only scope. `--base` is opt-in, so this ignores a
/// stale-but-present source that the `--base` form flags.
fn run_colocated_presence(repo: &TempRepo, language: &str) -> anyhow::Result<i32> {
    run([
        OsString::from("testing-conventions"),
        "unit".into(),
        "colocated-test".into(),
        repo.0.clone().into_os_string(),
        "--language".into(),
        language.into(),
    ])
}

const WIDGET_PY: &str = "def widget():\n    return 1\n";
const WIDGET_PY_TEST: &str =
    "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 1\n";

#[test]
fn python_modified_source_without_its_test_is_stale() {
    let repo = TempRepo::new("py-mod");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit the source only");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_modified_source_with_its_test_is_clean() {
    let repo = TempRepo::new("py-mod-clean");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("edit both");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_nested_source_is_reported_with_its_relative_path() {
    let repo = TempRepo::new("py-nested");
    repo.write("pkg/helper.py", "def helper():\n    return 1\n");
    repo.write(
        "pkg/helper_test.py",
        "def test_helper():\n    assert True\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("pkg/helper.py", "def helper():\n    return 2\n");
    repo.commit("edit nested source only");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["pkg/helper.py"]);
}

#[test]
fn python_deleted_source_without_deleting_its_test_is_stale() {
    let repo = TempRepo::new("py-del");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.remove("widget.py");
    repo.commit("delete the source only");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_deleting_source_and_test_together_is_clean() {
    let repo = TempRepo::new("py-del-both");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.remove("widget.py");
    repo.remove("widget_test.py");
    repo.commit("delete both");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_deleting_a_barrel_without_a_test_is_clean() {
    let repo = TempRepo::new("py-del-barrel");
    repo.write(
        "cli/interpret/__init__.py",
        "\"\"\"Interpret package.\"\"\"\n",
    );
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.remove("cli/interpret/__init__.py");
    repo.commit("delete the barrel");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_deleting_an_exempt_barrel_passes_base_after_dropping_its_entry() {
    let repo = TempRepo::new("py-del-exempt-barrel");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"cli/interpret/__init__.py\"\n\
         rules = [\"colocated-test\"]\nreason = \"package barrel; no logic to unit-test\"\n",
    );
    repo.write(
        "cli/interpret/__init__.py",
        "\"\"\"Interpret package.\"\"\"\n",
    );
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.remove("cli/interpret/__init__.py");
    repo.write("testing-conventions.toml", "");
    repo.commit("demolish the barrel");

    assert_eq!(
        run_co_change(&repo, "python", &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn python_added_source_is_not_a_subject() {
    let repo = TempRepo::new("py-add");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("fresh.py", "def fresh():\n    return 9\n");
    repo.commit("add a brand-new source");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_modifying_only_the_test_is_allowed() {
    let repo = TempRepo::new("py-test-only");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 1\n    assert widget() != 0\n",
    );
    repo.commit("strengthen the test only");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_modified_empty_file_is_not_a_subject() {
    let repo = TempRepo::new("py-empty");
    repo.write("pkg/__init__.py", "");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("pkg/__init__.py", "# a comment, still no code\n");
    repo.commit("touch the empty package init");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

const WIDGET_PY_COMMENTED: &str = "# widget helpers\ndef widget():\n    return 1\n";

#[test]
fn python_comment_only_edit_is_not_a_subject() {
    let repo = TempRepo::new("py-comment-only");
    repo.write("widget.py", WIDGET_PY_COMMENTED);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.py",
        "# widget utilities\ndef widget():\n    return 1\n",
    );
    repo.commit("reword the comment");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_removing_a_comment_is_not_a_subject() {
    let repo = TempRepo::new("py-comment-gone");
    repo.write("widget.py", WIDGET_PY_COMMENTED);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", WIDGET_PY);
    repo.commit("drop the comment");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_blank_line_only_edit_is_not_a_subject() {
    let repo = TempRepo::new("py-blank-line");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", "def widget():\n\n    return 1\n");
    repo.commit("space the body out");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_trailing_whitespace_only_edit_is_not_a_subject() {
    let repo = TempRepo::new("py-trailing-ws");
    repo.write("widget.py", "def widget():   \n    return 1   \n");
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", WIDGET_PY);
    repo.commit("strip trailing whitespace");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_comment_edit_carrying_a_code_change_is_stale() {
    let repo = TempRepo::new("py-comment-plus-code");
    repo.write("widget.py", WIDGET_PY_COMMENTED);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.py",
        "# widget utilities\ndef widget():\n    return 2\n",
    );
    repo.commit("reword the comment and change the value");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_docstring_edit_is_stale() {
    let repo = TempRepo::new("py-docstring");
    repo.write(
        "widget.py",
        "\"\"\"Widget helpers.\"\"\"\n\n\ndef widget():\n    return 1\n",
    );
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.py",
        "\"\"\"Widget utilities.\"\"\"\n\n\ndef widget():\n    return 1\n",
    );
    repo.commit("reword the docstring");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_string_literal_edit_is_stale() {
    let repo = TempRepo::new("py-string");
    repo.write("widget.py", "def widget():\n    return \"one\"\n");
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", "def widget():\n    return \"two\"\n");
    repo.commit("change the returned string");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_indentation_change_is_stale() {
    let repo = TempRepo::new("py-indent");
    repo.write(
        "widget.py",
        "def widget(flag):\n    if flag:\n        count = 1\n    return 1\n",
    );
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.py",
        "def widget(flag):\n    if flag:\n        count = 1\n        return 1\n",
    );
    repo.commit("pull the return into the branch");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_comment_edit_in_unparseable_source_is_stale() {
    let repo = TempRepo::new("py-unparseable");
    repo.write("widget.py", "def widget(:\n    return 1\n");
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.py", "# still broken\ndef widget(:\n    return 1\n");
    repo.commit("add a comment to the broken source");

    assert_eq!(stale(&repo, &base, Language::Python), vec!["widget.py"]);
}

#[test]
fn python_comment_only_edit_compares_against_the_merge_base() {
    let repo = TempRepo::new("py-merge-base");
    repo.write("widget.py", WIDGET_PY_COMMENTED);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    git(&repo.0, &["checkout", "-q", "-b", "trunk"]);
    git(&repo.0, &["checkout", "-q", "-b", "feature"]);

    repo.write(
        "widget.py",
        "# widget utilities\ndef widget():\n    return 1\n",
    );
    repo.commit("reword the comment");

    git(&repo.0, &["checkout", "-q", "trunk"]);
    repo.write(
        "widget.py",
        "# widget helpers\ndef widget():\n    return 2\n",
    );
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("advance trunk");
    git(&repo.0, &["checkout", "-q", "feature"]);

    assert!(stale(&repo, "trunk", Language::Python).is_empty());
}

#[test]
fn python_conftest_is_not_a_subject() {
    let repo = TempRepo::new("py-conftest");
    repo.write("conftest.py", "import pytest\n");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "conftest.py",
        "import pytest\n\n# a new fixture is coming\n",
    );
    repo.commit("edit conftest only");

    assert!(stale(&repo, &base, Language::Python).is_empty());
}

#[test]
fn python_subcommand_exits_nonzero_when_a_source_is_stale() {
    let repo = TempRepo::new("py-cli-red");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit the source only");

    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
}

#[test]
fn python_subcommand_exits_zero_when_every_change_co_changes() {
    let repo = TempRepo::new("py-cli-clean");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("edit both");

    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 0);
}

#[test]
fn python_a_co_change_exemption_lifts_a_stale_source() {
    let repo = TempRepo::new("py-exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"cli.py\"\nrules = [\"co-change\"]\n\
         reason = \"thin launcher; no logic to retest on each edit\"\n",
    );
    repo.write("cli.py", "def main():\n    return 0\n");
    repo.write(
        "cli_test.py",
        "from cli import main\n\n\ndef test_main():\n    assert main() == 0\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("cli.py", "def main():\n    return 1\n");
    repo.commit("edit the launcher, leave its test");

    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
    assert_eq!(
        run_co_change(&repo, "python", &base, Some("testing-conventions.toml")).unwrap(),
        0
    );
}

#[test]
fn a_stale_exempt_entry_is_an_error() {
    let repo = TempRepo::new("py-stale-exempt");
    repo.write(
        "testing-conventions.toml",
        "[[python.exempt]]\npath = \"ghost.py\"\nrules = [\"co-change\"]\nreason = \"gone\"\n",
    );
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit source only");

    assert!(run_co_change(&repo, "python", &base, Some("testing-conventions.toml")).is_err());
}

#[test]
fn typescript_modified_source_without_its_test_is_stale() {
    let repo = TempRepo::new("ts-mod");
    repo.write("widget.ts", "export const widget = () => 1;\n");
    repo.write(
        "widget.test.ts",
        "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(1));\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", "export const widget = () => 2;\n");
    repo.commit("edit the source only");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["widget.ts"]);
}

#[test]
fn typescript_modified_source_with_its_test_is_clean() {
    let repo = TempRepo::new("ts-mod-clean");
    repo.write("widget.ts", "export const widget = () => 1;\n");
    repo.write(
        "widget.test.ts",
        "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(1));\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", "export const widget = () => 2;\n");
    repo.write(
        "widget.test.ts",
        "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(2));\n",
    );
    repo.commit("edit both");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

#[test]
fn typescript_deleting_a_barrel_without_a_test_is_clean() {
    let repo = TempRepo::new("ts-del-barrel");
    repo.write("cli/interpret/index.ts", "export * from './widget';\n");
    repo.write(
        "cli/interpret/widget.ts",
        "export const widget = () => 1;\n",
    );
    repo.write(
        "cli/interpret/widget.test.ts",
        "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(1));\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.remove("cli/interpret/index.ts");
    repo.commit("delete the barrel");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

const TS_WIDGET: &str = "export const widget = () => 1;\n";
const TS_WIDGET_TEST: &str =
    "import { widget } from './widget';\nit('works', () => expect(widget()).toBe(1));\n";

#[test]
fn typescript_modified_type_only_module_is_not_a_subject() {
    let repo = TempRepo::new("ts-type-only");
    repo.write("widget.ts", TS_WIDGET);
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.write("aliases.ts", "export type Alias = string;\n");
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "aliases.ts",
        "export type Alias = string;\nexport type Alias2 = number;\n",
    );
    repo.commit("extend the type-only module");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

#[test]
fn typescript_modified_module_mixing_types_and_runtime_is_a_subject() {
    let repo = TempRepo::new("ts-mixed");
    repo.write("mixed.ts", "export type Alias = string;\n");
    repo.write(
        "mixed.test.ts",
        "it('nothing yet', () => expect(1).toBe(1));\n",
    );
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "mixed.ts",
        "export type Alias = string;\nexport const build = () => 1;\n",
    );
    repo.commit("add a runtime export");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["mixed.ts"]);
}

#[test]
fn typescript_modified_runtime_module_without_a_colocated_test_is_still_stale() {
    let repo = TempRepo::new("ts-loose");
    repo.write("loose.ts", "export const loose = () => 1;\n");
    repo.commit("base");
    let base = repo.head();

    repo.write("loose.ts", "export const loose = () => 2;\n");
    repo.commit("edit the untested runtime module");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["loose.ts"]);
}

#[test]
fn typescript_deleting_a_type_only_module_is_clean() {
    let repo = TempRepo::new("ts-del-type-only");
    repo.write("widget.ts", TS_WIDGET);
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.write("aliases.ts", "export type Alias = string;\n");
    repo.commit("base");
    let base = repo.head();

    repo.remove("aliases.ts");
    repo.commit("delete the type-only module");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

const TS_WIDGET_COMMENTED: &str = "// widget factory\nexport const widget = () => 1;\n";

#[test]
fn typescript_line_comment_only_edit_is_not_a_subject() {
    let repo = TempRepo::new("ts-comment-only");
    repo.write("widget.ts", TS_WIDGET_COMMENTED);
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.ts",
        "// widget builder\nexport const widget = () => 1;\n",
    );
    repo.commit("reword the comment");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

#[test]
fn typescript_removing_a_block_comment_is_not_a_subject() {
    let repo = TempRepo::new("ts-block-comment");
    repo.write(
        "widget.ts",
        "/* widget factory\n   used by the CLI */\nexport const widget = () => 1;\n",
    );
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", TS_WIDGET);
    repo.commit("drop the block comment");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

#[test]
fn typescript_blank_line_only_edit_is_not_a_subject() {
    let repo = TempRepo::new("ts-blank-line");
    repo.write("widget.ts", TS_WIDGET);
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", "\nexport const widget = () => 1;\n\n");
    repo.commit("space the module out");

    assert!(stale(&repo, &base, Language::TypeScript).is_empty());
}

#[test]
fn typescript_comment_edit_carrying_a_code_change_is_stale() {
    let repo = TempRepo::new("ts-comment-plus-code");
    repo.write("widget.ts", TS_WIDGET_COMMENTED);
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.ts",
        "// widget builder\nexport const widget = () => 2;\n",
    );
    repo.commit("reword the comment and change the value");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["widget.ts"]);
}

#[test]
fn typescript_template_literal_edit_is_stale() {
    let repo = TempRepo::new("ts-template");
    repo.write("widget.ts", "export const widget = () => `one`;\n");
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", "export const widget = () => `two`;\n");
    repo.commit("change the produced string");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["widget.ts"]);
}

#[test]
fn typescript_string_literal_edit_is_stale() {
    let repo = TempRepo::new("ts-string");
    repo.write("widget.ts", "export const widget = () => 'one';\n");
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write("widget.ts", "export const widget = () => 'two';\n");
    repo.commit("change the returned string");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["widget.ts"]);
}

#[test]
fn typescript_comment_edit_in_unparseable_source_is_stale() {
    let repo = TempRepo::new("ts-unparseable");
    repo.write("widget.ts", "export const widget = (() => 1;\n");
    repo.write("widget.test.ts", TS_WIDGET_TEST);
    repo.commit("base");
    let base = repo.head();

    repo.write(
        "widget.ts",
        "// still broken\nexport const widget = (() => 1;\n",
    );
    repo.commit("add a comment to the broken source");

    assert_eq!(stale(&repo, &base, Language::TypeScript), vec!["widget.ts"]);
}

#[test]
fn an_unknown_base_ref_is_an_error() {
    let repo = TempRepo::new("bad-base");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");

    assert!(
        stale_sources(&repo.0, "no-such-ref", Language::Python, &BTreeSet::new()).is_err(),
        "an unresolvable base ref must error"
    );
}

#[test]
fn co_change_rejects_rust() {
    let repo = TempRepo::new("rust-reject");
    repo.write("lib.rs", "pub fn f() {}\n");
    repo.commit("base");
    let base = repo.head();

    let err = run_co_change(&repo, "rust", &base, None).unwrap_err();
    assert!(err.to_string().contains("inline"), "got: {err}");
}

#[test]
fn base_adds_co_change_on_top_of_presence() {
    let repo = TempRepo::new("base-additive");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.commit("edit the source only");

    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
    assert_eq!(run_colocated_presence(&repo, "python").unwrap(), 0);
}

#[test]
fn base_still_enforces_tree_wide_presence() {
    let repo = TempRepo::new("base-presence");
    repo.write("widget.py", WIDGET_PY);
    repo.write("widget_test.py", WIDGET_PY_TEST);
    repo.write("orphan.py", "def orphan():\n    return 9\n");
    repo.commit("base");
    let base = repo.head();
    repo.write("widget.py", "def widget():\n    return 2\n");
    repo.write(
        "widget_test.py",
        "from widget import widget\n\n\ndef test_widget():\n    assert widget() == 2\n",
    );
    repo.commit("edit widget and its test together");

    assert_eq!(run_co_change(&repo, "python", &base, None).unwrap(), 1);
}
