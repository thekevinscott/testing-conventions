"""Unit tests for the counting rules: scope, test-file exclusion, per-line classification."""
import textwrap

import pytest

from code_lines import area_of, classify, is_test_file


def kinds(path, source):
    return classify(path, textwrap.dedent(source).lstrip("\n"))


@pytest.mark.parametrize(
    "path,area",
    [
        ("packages/rust/src/lib.rs", "packages/rust/src"),
        ("packages/python/python/testing_conventions/config/detect.py", "packages/python/python"),
        ("packages/node/src/mutation/run-stryker.ts", "packages/node/src"),
        ("packages/node/scripts/postinstall.mjs", "packages/node/scripts"),
        ("internals/checks/src/checks/cli.py", "internals/checks/src"),
        ("internals/move-major-tag/src/move_major_tag.py", "internals/move-major-tag/src"),
    ],
)
def test_area_is_the_package_source_root(path, area):
    assert area_of(path) == area


@pytest.mark.parametrize(
    "path",
    [
        "packages/node/vite.config.ts",
        "packages/node/eslint.config.js",
        "packages/node/vitest.e2e.config.ts",
    ],
)
def test_a_source_file_at_a_package_root_is_build_config(path):
    assert area_of(path) is None


@pytest.mark.parametrize(
    "path",
    [
        "docs/.vitepress/config.ts",
        ".github/selftest/python/src/pkg.py",
        "packages/rust/Cargo.toml",
        "packages/node/src/index.d.ts.map",
    ],
)
def test_out_of_scope_paths_have_no_area(path):
    assert area_of(path) is None


@pytest.mark.parametrize(
    "path",
    [
        "packages/rust/tests/lint_test.rs",
        "packages/rust/tests/fixtures/isolation/unit/red/src/lib.rs",
        "internals/checks/tests/e2e/gates_wired_e2e_test.py",
        "internals/checks/src/checks/gates_wired/cli_test.py",
        "internals/detect/src/detect_test.py",
        "packages/python/python/testing_conventions/config/test_detect.py",
        "packages/node/src/mutation/to-normalized.test.ts",
        "packages/node/scripts/build.test.mjs",
    ],
)
def test_test_files_are_excluded_wherever_they_live(path):
    assert is_test_file(path)


@pytest.mark.parametrize(
    "path",
    [
        "internals/checks/src/checks/gates_wired/cli.py",
        "packages/node/src/mutation/to-normalized.ts",
        "packages/rust/src/lint.rs",
        "packages/python/python/testing_conventions/latest.py",
    ],
)
def test_source_siblings_of_test_files_are_not_excluded(path):
    assert not is_test_file(path)


def test_an_excluded_file_reports_every_line_as_test():
    assert kinds("internals/detect/src/detect_test.py", '''
        """A docstring."""
        def test_one():
            assert True
    ''') == ["test"] * 3


def test_python_docstrings_count_as_comments():
    assert kinds("internals/checks/src/checks/cli.py", '''
        """Module docstring.

        Second paragraph.
        """
        def f():
            """One-liner."""
            return 1
    ''') == ["comment", "blank", "comment", "comment", "code", "comment", "code"]


def test_python_comments_and_blanks_are_not_code():
    assert kinds("internals/detect/src/detect.py", """
        # a leading comment
        x = 1  # a trailing comment

        y = 2
    """) == ["comment", "code", "blank", "code"]


def test_a_python_string_that_is_not_a_docstring_is_code():
    assert kinds("internals/detect/src/detect.py", '''
        TEMPLATE = """
        line one
        """
    ''') == ["code", "code", "code"]


def test_a_blank_line_inside_a_python_docstring_stays_blank():
    assert kinds("internals/detect/src/detect.py", '''
        """First.

        Third.
        """
    ''') == ["comment", "blank", "comment", "comment"]


def test_rust_line_and_block_comments_are_not_code():
    assert kinds("packages/rust/src/lib.rs", """
        //! Module doc.
        /// Item doc.
        pub fn f() -> u8 {
            // inner
            /* block
               continues */
            1
        }
    """) == ["comment", "comment", "code", "comment", "comment", "comment", "code", "code"]


def test_rust_nested_block_comments_close_at_the_outer_delimiter():
    assert kinds("packages/rust/src/lib.rs", """
        /* outer /* inner */ still comment */
        pub fn f() {}
    """) == ["comment", "code"]


def test_the_inline_cfg_test_region_is_excluded_by_brace_matching():
    assert kinds("packages/rust/src/tiers.rs", """
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
    """) == ["code", "code", "code", "blank"] + ["test"] * 7


def test_braces_inside_a_test_string_do_not_close_the_region():
    assert kinds("packages/rust/src/lib.rs", """
        pub fn f() {}

        #[cfg(test)]
        mod tests {
            #[test]
            fn t() {
                assert_eq!(format!("{}", 1), "}");
            }
        }
        pub fn after_the_region() {}
    """) == ["code", "blank"] + ["test"] * 7 + ["code"]


def test_cfg_test_inside_a_string_literal_does_not_open_a_region():
    assert kinds("packages/rust/src/one_function.rs", r"""
        pub const SAMPLE: &str = "#[cfg(test)]\nmod tests { }\n";
        pub fn f() {}
    """) == ["code", "code"]


def test_an_escaped_newline_in_a_string_keeps_line_accounting_aligned():
    assert kinds("packages/rust/src/lib.rs", r"""
        pub const MESSAGE: &str = "a long message that wraps \
             onto the next source line";
        // a comment that must still read as a comment
        pub fn f() {}
    """) == ["code", "code", "comment", "code"]


def test_a_rust_raw_string_hides_quotes_and_slashes():
    assert kinds("packages/rust/src/config.rs", r"""
        pub const P: &str = r#"a "quoted" // not a comment"#;
        pub fn f() {}
    """) == ["code", "code"]


def test_a_lifetime_is_not_an_unterminated_char_literal():
    assert kinds("packages/rust/src/lint.rs", r"""
        pub fn f<'a>(s: &'a str) -> &'a str {
            let sep = '/';
            // still a comment
            s
        }
    """) == ["code", "code", "comment", "code", "code"]


def test_typescript_comments_and_template_literals():
    assert kinds("packages/node/src/mutation/run-stryker.ts", """
        // a comment
        const q = `a template
        spanning // lines`;
        /* block */
        export const n = 1;
    """) == ["comment", "code", "code", "comment", "code"]
