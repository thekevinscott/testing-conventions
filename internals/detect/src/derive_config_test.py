from pathlib import Path

import pytest

import derive_config


@pytest.mark.parametrize("override", ["./custom.toml", "vendor/custom.toml"])
def test_derive_config_returns_an_explicit_override_verbatim(tmp_path, in_dir, write, override):
    in_dir(tmp_path)
    write(tmp_path / "packages" / "app" / "testing-conventions.toml")
    assert derive_config.derive_config(Path("packages/app"), override) == override


def test_derive_config_discovers_the_file_at_the_package_root(tmp_path, in_dir, write):
    in_dir(tmp_path)
    write(tmp_path / "packages" / "app" / "testing-conventions.toml")
    derived = derive_config.derive_config(Path("packages/app"), "testing-conventions.toml")
    assert derived == str(Path("packages/app/testing-conventions.toml"))


def test_derive_config_falls_back_to_the_default_when_the_package_has_none(tmp_path, in_dir):
    in_dir(tmp_path)
    (tmp_path / "packages" / "app").mkdir(parents=True)
    derived = derive_config.derive_config(Path("packages/app"), "testing-conventions.toml")
    assert derived == "testing-conventions.toml"
