"""Integration tests: the real orchestration over a mocked git and filesystem boundary."""
import textwrap
from pathlib import Path
from unittest.mock import patch

import code_lines

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
                assert_eq!(f(), 1);
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
}


def tree():
    return {path: textwrap.dedent(body).lstrip("\n") for path, body in TREE.items()}


def collected():
    files = tree()
    with (
        patch.object(code_lines, "tracked_paths", return_value=sorted(files)),
        patch.object(code_lines, "read_source", side_effect=lambda root, path: files[path]),
    ):
        return code_lines.collect(Path("/repo"))


def test_collect_drops_out_of_scope_paths():
    assert "packages/node/vite.config.ts" not in collected()


def test_collect_keeps_test_files_so_their_lines_are_reported():
    assert "packages/rust/tests/lib_test.rs" in collected()


def test_tally_groups_by_area_and_splits_the_four_kinds():
    assert code_lines.tally(collected()) == {
        "packages/rust/src": {"code": 3, "comment": 1, "blank": 1, "test": 7},
        "packages/rust/tests": {"test": 2},
        "internals/checks/src": {"code": 1, "comment": 1, "test": 2},
    }


def test_render_sorts_by_code_and_totals_every_column():
    assert code_lines.render(code_lines.tally(collected())) == textwrap.dedent("""\
        | area | code | comment | blank | test |
        | --- | ---: | ---: | ---: | ---: |
        | packages/rust/src | 3 | 1 | 1 | 7 |
        | internals/checks/src | 1 | 1 | 0 | 2 |
        | packages/rust/tests | 0 | 0 | 0 | 2 |
        | **total** | **4** | 2 | 1 | 11 |""")


def test_main_prints_the_table_and_succeeds(capsys):
    files = tree()
    with (
        patch.object(code_lines, "repo_root", return_value=Path("/repo")),
        patch.object(code_lines, "tracked_paths", return_value=sorted(files)),
        patch.object(code_lines, "read_source", side_effect=lambda root, path: files[path]),
    ):
        assert code_lines.main() == 0
    assert "| **total** | **4** | 2 | 1 | 11 |" in capsys.readouterr().out
