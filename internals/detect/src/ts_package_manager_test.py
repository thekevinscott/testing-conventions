import ts_package_manager


def test_package_manager_from_field_extracts_the_name():
    assert ts_package_manager._package_manager_from_field("pnpm@8.6.0") == "pnpm"


def test_package_manager_from_field_empty_is_none():
    assert ts_package_manager._package_manager_from_field("") is None


def test_ts_package_manager_prefers_the_declared_field(tmp_path, write):
    write(tmp_path / "package.json", '{"packageManager": "yarn@4.0.0"}')
    write(tmp_path / "pnpm-lock.yaml")
    assert ts_package_manager.ts_package_manager(tmp_path) == "yarn"


def test_ts_package_manager_reads_a_pnpm_lockfile(tmp_path, write):
    write(tmp_path / "pnpm-lock.yaml")
    assert ts_package_manager.ts_package_manager(tmp_path) == "pnpm"


def test_ts_package_manager_reads_an_npm_lockfile(tmp_path, write):
    write(tmp_path / "package-lock.json")
    assert ts_package_manager.ts_package_manager(tmp_path) == "npm"


def test_ts_package_manager_defaults_to_pnpm(tmp_path):
    assert ts_package_manager.ts_package_manager(tmp_path) == "pnpm"
