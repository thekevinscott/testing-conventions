import pytest

import manifest


@pytest.mark.parametrize("name", ["package.json", "pyproject.toml", "Cargo.toml"])
def test_has_manifest_recognizes_each_manifest(tmp_path, write, name):
    write(tmp_path / name)
    assert manifest.has_manifest(tmp_path) is True


def test_has_manifest_ignores_a_manifest_in_a_subdirectory(tmp_path, write):
    write(tmp_path / "sub" / "pyproject.toml")
    assert manifest.has_manifest(tmp_path) is False


def test_has_lockfile_finds_the_named_file(tmp_path, write):
    write(tmp_path / "pnpm-lock.yaml")
    assert manifest.has_lockfile(tmp_path, "pnpm-lock.yaml") is True


def test_has_lockfile_is_false_for_another_name(tmp_path, write):
    write(tmp_path / "pnpm-lock.yaml")
    assert manifest.has_lockfile(tmp_path, "package-lock.json") is False


def test_has_lockfile_is_false_for_a_directory_of_that_name(tmp_path):
    (tmp_path / "pnpm-lock.yaml").mkdir()
    assert manifest.has_lockfile(tmp_path, "pnpm-lock.yaml") is False
