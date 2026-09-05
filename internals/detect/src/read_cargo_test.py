import read_cargo


def test_read_cargo_parses_the_manifest(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert read_cargo.read_cargo(tmp_path) == {"package": {"name": "c"}}


def test_read_cargo_is_empty_when_absent(tmp_path):
    assert read_cargo.read_cargo(tmp_path) == {}


def test_read_cargo_is_empty_when_malformed(tmp_path, write):
    write(tmp_path / "Cargo.toml", "[package\n")
    assert read_cargo.read_cargo(tmp_path) == {}
