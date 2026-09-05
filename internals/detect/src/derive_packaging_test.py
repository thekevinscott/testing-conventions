import derive_packaging


def test_derive_packaging_builds_a_python_project_with_uv(tmp_path, write):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert derive_packaging.derive_packaging(tmp_path, "python", tmp_path) == "uv build"


def test_derive_packaging_is_empty_for_a_tool_only_pyproject(tmp_path, write):
    write(tmp_path / "pyproject.toml", "[tool.ruff]\n")
    assert derive_packaging.derive_packaging(tmp_path, "python", tmp_path) == ""


def test_derive_packaging_packs_a_typescript_package_with_its_manager(tmp_path, write):
    write(tmp_path / "package.json", '{"packageManager": "npm@10.0.0"}')
    derived = derive_packaging.derive_packaging(tmp_path, "typescript", tmp_path)
    assert derived == "npm pack --pack-destination dist"


def test_derive_packaging_packages_a_standalone_crate(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert derive_packaging.derive_packaging(tmp_path, "rust", tmp_path) == "cargo package"


def test_derive_packaging_redirects_a_workspace_members_target_dir(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    derived = derive_packaging.derive_packaging(member, "rust", tmp_path)
    assert derived == "cargo package --target-dir target"


def test_derive_packaging_is_empty_for_a_workspace_only_manifest(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    assert derive_packaging.derive_packaging(tmp_path, "rust", tmp_path) == ""


def test_derive_packaging_is_empty_for_a_language_the_table_does_not_name(tmp_path, write):
    write(tmp_path / "pyproject.toml", '[project]\nname = "p"\n')
    assert derive_packaging.derive_packaging(tmp_path, "", tmp_path) == ""
