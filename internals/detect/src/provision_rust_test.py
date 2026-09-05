import provision_rust


def test_provision_rust_for_a_cargo_manifest(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert provision_rust.provision_rust(tmp_path) == "true"


def test_provision_rust_for_a_maturin_backend(tmp_path, write):
    write(tmp_path / "pyproject.toml", '[build-system]\nbuild-backend = "maturin"\n')
    assert provision_rust.provision_rust(tmp_path) == "true"


def test_provision_rust_ignores_another_build_backend(tmp_path, write):
    write(tmp_path / "pyproject.toml", '[build-system]\nbuild-backend = "hatchling.build"\n')
    assert provision_rust.provision_rust(tmp_path) == "false"


def test_provision_rust_for_a_napi_key(tmp_path, write):
    write(tmp_path / "package.json", '{"napi": {"name": "core"}}')
    assert provision_rust.provision_rust(tmp_path) == "true"


def test_provision_rust_for_a_napi_cli_devdependency(tmp_path, write):
    write(tmp_path / "package.json", '{"devDependencies": {"@napi-rs/cli": "^2.0.0"}}')
    assert provision_rust.provision_rust(tmp_path) == "true"


def test_provision_rust_is_false_for_a_plain_node_package(tmp_path, write):
    write(tmp_path / "package.json", '{"devDependencies": {"vitest": "^3.0.0"}}')
    assert provision_rust.provision_rust(tmp_path) == "false"
