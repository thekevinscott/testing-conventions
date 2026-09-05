import read_package_json


def test_read_package_json_parses_the_manifest(tmp_path, write):
    write(tmp_path / "package.json", '{"packageManager": "pnpm@9.1.0"}')
    assert read_package_json.read_package_json(tmp_path) == {"packageManager": "pnpm@9.1.0"}


def test_read_package_json_is_empty_when_absent(tmp_path):
    assert read_package_json.read_package_json(tmp_path) == {}


def test_read_package_json_is_empty_when_malformed(tmp_path, write):
    write(tmp_path / "package.json", "{not json")
    assert read_package_json.read_package_json(tmp_path) == {}
