"""End-to-end tests: the real script, over a real git repository, with no mocks."""
import subprocess
import sys
import textwrap
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[2] / "src" / "code_lines.py"

TREE = {
    "packages/rust/src/lib.rs": """
        //! Doc.
        pub fn f() -> u8 {
            1
        }

        #[cfg(test)]
        mod tests {
            #[test]
            fn t() {
                assert_eq!(f(), "}");
            }
        }
    """,
    "packages/rust/tests/lib_test.rs": """
        #[test]
        fn integration() {}
    """,
    "internals/checks/src/checks/cli.py": '''
        """Docstring."""
        X = 1
    ''',
    "internals/checks/src/checks/cli_test.py": """
        def test_x():
            assert True
    """,
    "packages/node/vite.config.ts": """
        export default {};
    """,
    "docs/.vitepress/config.ts": """
        export default {};
    """,
}


def git(repo, *args):
    return subprocess.run(
        ["git", "-C", str(repo), *args], capture_output=True, text=True, check=True
    ).stdout


@pytest.fixture
def repo(tmp_path):
    git(tmp_path, "init", "-q", "-b", "main")
    git(tmp_path, "config", "user.email", "test@example.com")
    git(tmp_path, "config", "user.name", "test")
    for path, body in TREE.items():
        target = tmp_path / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(textwrap.dedent(body).lstrip("\n"))
    git(tmp_path, "add", "-A")
    git(tmp_path, "commit", "-q", "-m", "fixture")
    return tmp_path


def run(cwd):
    result = subprocess.run(
        [sys.executable, str(SCRIPT)], cwd=str(cwd), capture_output=True, text=True
    )
    assert result.returncode == 0, result.stderr
    return result.stdout


def test_the_table_reports_every_area(repo):
    assert run(repo).splitlines() == [
        "| area | code | comment | blank | test |",
        "| --- | ---: | ---: | ---: | ---: |",
        "| packages/rust/src | 3 | 1 | 1 | 7 |",
        "| internals/checks/src | 1 | 1 | 0 | 2 |",
        "| packages/rust/tests | 0 | 0 | 0 | 2 |",
        "| **total** | **4** | 2 | 1 | 11 |",
    ]


def test_it_runs_from_any_directory_in_the_repository(repo):
    assert run(repo / "packages" / "rust" / "src") == run(repo)


def test_running_the_tool_leaves_the_working_tree_clean(repo):
    before = git(repo, "status", "--porcelain")
    run(repo)
    assert git(repo, "status", "--porcelain") == before == ""
