import pytest

import file_presence


@pytest.mark.parametrize(
    ("language", "filename"),
    [
        ("python", "mod.py"),
        ("typescript", "mod.ts"),
        ("typescript", "view.tsx"),
        ("typescript", "mod.mts"),
        ("typescript", "mod.cts"),
    ],
)
def test_has_source_finds_a_language_file_at_any_depth(tmp_path, write, language, filename):
    write(tmp_path / "a" / "b" / filename)
    assert file_presence.has_source(tmp_path, language) is True


def test_has_source_ignores_another_languages_files(tmp_path, write):
    write(tmp_path / "mod.ts")
    assert file_presence.has_source(tmp_path, "python") is False


def test_has_source_is_false_for_an_empty_tree(tmp_path):
    assert file_presence.has_source(tmp_path, "typescript") is False


@pytest.mark.parametrize(("language", "filename"), [("python", "e2e/version_test.py"), ("typescript", "e2e/cli.test.ts")])
def test_has_source_ignores_the_tests_tier_beneath_the_root(tmp_path, write, language, filename):
    write(tmp_path / "tests" / filename)
    assert file_presence.has_source(tmp_path, language) is False


def test_has_source_counts_a_tests_directory_nested_below_the_root(tmp_path, write):
    write(tmp_path / "pkg" / "tests" / "mod.py")
    assert file_presence.has_source(tmp_path, "python") is True
