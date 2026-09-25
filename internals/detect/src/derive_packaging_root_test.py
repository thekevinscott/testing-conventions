from pathlib import Path

import derive_packaging_root


def test_derive_packaging_root_python_is_the_package_dist():
    derived = derive_packaging_root.derive_packaging_root(Path("packages/py"), "python")
    assert derived == "packages/py/dist"


def test_derive_packaging_root_typescript_is_the_package_dist():
    derived = derive_packaging_root.derive_packaging_root(Path("packages/ts"), "typescript")
    assert derived == "packages/ts/dist"


def test_derive_packaging_root_rust_is_the_crate_output():
    derived = derive_packaging_root.derive_packaging_root(Path("packages/rust"), "rust")
    assert derived == "packages/rust/target/package"


def test_derive_packaging_root_of_the_repo_root_package():
    derived = derive_packaging_root.derive_packaging_root(Path("."), "python")
    assert derived == "./dist"


def test_derive_packaging_root_falls_back_to_the_dist_with_no_primary_language():
    derived = derive_packaging_root.derive_packaging_root(Path("packages/py"), "")
    assert derived == "packages/py/dist"
