import pytest

import has_dist


@pytest.mark.parametrize(
    "artifact",
    ["pkg-1.0-py3-none-any.whl", "pkg-1.0.tar.gz", "pkg-1.0.tgz", "pkg-1.0.crate"],
)
def test_has_dist_recognizes_each_built_distribution(tmp_path, write, artifact):
    write(tmp_path / "dist" / artifact)
    assert has_dist.has_dist(tmp_path) is True


def test_has_dist_ignores_an_unrecognized_file_in_dist(tmp_path, write):
    write(tmp_path / "dist" / "README.md")
    assert has_dist.has_dist(tmp_path) is False


def test_has_dist_is_false_without_a_dist_directory(tmp_path, write):
    write(tmp_path / "pkg-1.0.tar.gz")
    assert has_dist.has_dist(tmp_path) is False


def test_has_dist_is_false_when_dist_is_a_file(tmp_path, write):
    write(tmp_path / "dist")
    assert has_dist.has_dist(tmp_path) is False
