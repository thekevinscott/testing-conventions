import pytest

import primary_language


@pytest.mark.parametrize(
    ("manifest", "expected"),
    [("pyproject.toml", "python"), ("package.json", "typescript"), ("Cargo.toml", "rust")],
)
def test_primary_language_reads_each_manifest(tmp_path, write, manifest, expected):
    write(tmp_path / manifest)
    assert primary_language.primary_language(tmp_path) == expected


def test_primary_language_prefers_python_for_a_pyo3_binding(tmp_path, write):
    write(tmp_path / "pyproject.toml")
    write(tmp_path / "Cargo.toml")
    assert primary_language.primary_language(tmp_path) == "python"


def test_primary_language_prefers_typescript_for_a_napi_binding(tmp_path, write):
    write(tmp_path / "package.json")
    write(tmp_path / "Cargo.toml")
    assert primary_language.primary_language(tmp_path) == "typescript"


def test_primary_language_is_empty_without_a_manifest(tmp_path):
    assert primary_language.primary_language(tmp_path) == ""
