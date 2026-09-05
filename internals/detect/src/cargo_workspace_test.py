import cargo_workspace


def test_cargo_workspace_root_finds_the_owning_ancestor(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert cargo_workspace.cargo_workspace_root(member, tmp_path) == tmp_path.resolve()


def test_cargo_workspace_root_is_none_without_a_workspace_ancestor(tmp_path, write):
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert cargo_workspace.cargo_workspace_root(member, tmp_path) is None


def test_cargo_workspace_root_ignores_the_package_roots_own_workspace_table(tmp_path, write):
    root = tmp_path / "crate"
    write(root / "Cargo.toml", '[package]\nname = "c"\n\n[workspace]\n')
    assert cargo_workspace.cargo_workspace_root(root, tmp_path) is None


def test_cargo_workspace_root_is_none_when_the_package_is_the_repo_root(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[workspace]\n')
    assert cargo_workspace.cargo_workspace_root(tmp_path, tmp_path) is None


def test_cargo_workspace_root_prefers_the_nearest_workspace(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[workspace]\n')
    inner = tmp_path / "sub"
    write(inner / "Cargo.toml", '[workspace]\n')
    member = inner / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert cargo_workspace.cargo_workspace_root(member, tmp_path) == inner.resolve()


def test_cargo_workspace_root_ignores_a_workspace_above_the_repo_root(tmp_path, write):
    write(tmp_path / "Cargo.toml", "[workspace]\n")
    member = tmp_path / "repo" / "crate"
    write(member / "Cargo.toml", '[package]\nname = "c"\n')
    assert cargo_workspace.cargo_workspace_root(member, tmp_path / "repo") is None


def test_cargo_workspace_root_finds_a_workspace_above_a_package_outside_the_repo(tmp_path, write):
    write(tmp_path / "Cargo.toml", "[workspace]\n")
    member = tmp_path / "aside" / "crate"
    write(member / "Cargo.toml", '[package]\nname = "c"\n')
    (tmp_path / "repo").mkdir()
    assert cargo_workspace.cargo_workspace_root(member, tmp_path / "repo") == tmp_path.resolve()


def test_is_workspace_member_for_a_crate_under_a_workspace(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[workspace]\nmembers = ["crates/*"]\n')
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert cargo_workspace.is_workspace_member(member, tmp_path) is True


def test_is_workspace_member_is_false_for_a_standalone_crate(tmp_path, write):
    member = tmp_path / "crates" / "core"
    write(member / "Cargo.toml", '[package]\nname = "core"\n')
    assert cargo_workspace.is_workspace_member(member, tmp_path) is False


def test_is_workspace_member_is_false_for_a_workspace_root_package(tmp_path, write):
    root = tmp_path / "crate"
    write(root / "Cargo.toml", '[package]\nname = "c"\n\n[workspace]\n')
    assert cargo_workspace.is_workspace_member(root, tmp_path) is False


def test_is_workspace_member_is_false_when_the_package_is_the_repo_root(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[workspace]\n')
    assert cargo_workspace.is_workspace_member(tmp_path, tmp_path) is False
