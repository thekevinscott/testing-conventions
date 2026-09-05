import read_pyproject


def test_read_pyproject_parses_the_manifest(tmp_path, write):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert read_pyproject.read_pyproject(tmp_path) == {"project": {"name": "p"}}


def test_read_pyproject_is_empty_when_absent(tmp_path):
    assert read_pyproject.read_pyproject(tmp_path) == {}


def test_read_pyproject_is_empty_when_malformed(tmp_path, write):
    write(tmp_path / "pyproject.toml", "[project\n")
    assert read_pyproject.read_pyproject(tmp_path) == {}
