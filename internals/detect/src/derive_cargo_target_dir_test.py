from pathlib import Path

import derive_cargo_target_dir


def test_derive_cargo_target_dir_standalone_uses_the_package_root():
    derived = derive_cargo_target_dir.derive_cargo_target_dir(Path("packages/rust"), None)
    assert derived == "packages/rust/target"


def test_derive_cargo_target_dir_workspace_member_uses_the_workspace_root():
    derived = derive_cargo_target_dir.derive_cargo_target_dir(Path("packages/rust"), Path("."))
    assert derived == "./target"
