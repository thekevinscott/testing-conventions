import any_source_match


def test_finds_a_match_at_any_depth(tmp_path, write):
    write(tmp_path / "a" / "b" / "mod.py")
    assert any_source_match.any_source_match(tmp_path, ("*.py",)) is True


def test_skips_the_tests_directory_beneath_the_root(tmp_path, write):
    write(tmp_path / "tests" / "e2e" / "mod_test.py")
    assert any_source_match.any_source_match(tmp_path, ("*.py",)) is False


def test_counts_a_tests_directory_nested_deeper(tmp_path, write):
    write(tmp_path / "pkg" / "tests" / "mod.py")
    assert any_source_match.any_source_match(tmp_path, ("*.py",)) is True


def test_checks_every_glob(tmp_path, write):
    write(tmp_path / "tests" / "a.ts")
    write(tmp_path / "b.tsx")
    assert any_source_match.any_source_match(tmp_path, ("*.ts", "*.tsx")) is True


def test_is_false_for_an_empty_tree(tmp_path):
    assert any_source_match.any_source_match(tmp_path, ("*.py",)) is False
