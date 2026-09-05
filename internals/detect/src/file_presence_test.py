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


def test_has_rust_crate_needs_a_manifest_and_real_source(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "src" / "lib.rs", "pub fn f() {}\n")
    assert file_presence.has_rust_crate(tmp_path) is True


def test_has_rust_crate_is_false_for_a_manifest_with_no_source(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert file_presence.has_rust_crate(tmp_path) is False


def test_has_rust_crate_is_false_for_source_with_no_manifest(tmp_path, write):
    write(tmp_path / "src" / "lib.rs", "pub fn f() {}\n")
    assert file_presence.has_rust_crate(tmp_path) is False
