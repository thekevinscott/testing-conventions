import derive_package_root


def test_derive_package_root_finds_the_nearest_manifest_above_the_scan(tmp_path, write):
    write(tmp_path / "packages" / "app" / "pyproject.toml")
    scan = tmp_path / "packages" / "app" / "src"
    scan.mkdir()
    derived = derive_package_root.derive_package_root(scan, tmp_path)
    assert derived == (tmp_path / "packages" / "app").resolve()


def test_derive_package_root_prefers_the_scan_root_over_an_ancestor(tmp_path, write):
    write(tmp_path / "pyproject.toml")
    write(tmp_path / "packages" / "app" / "package.json")
    scan = tmp_path / "packages" / "app"
    assert derive_package_root.derive_package_root(scan, tmp_path) == scan.resolve()


def test_derive_package_root_falls_back_to_the_repo_root_with_no_manifest(tmp_path):
    scan = tmp_path / "packages" / "app" / "src"
    scan.mkdir(parents=True)
    assert derive_package_root.derive_package_root(scan, tmp_path) == tmp_path.resolve()


def test_derive_package_root_finds_a_manifest_above_a_scan_outside_the_repo(tmp_path, write):
    write(tmp_path / "aside" / "pyproject.toml")
    scan = tmp_path / "aside" / "src"
    scan.mkdir()
    (tmp_path / "repo").mkdir()
    derived = derive_package_root.derive_package_root(scan, tmp_path / "repo")
    assert derived == (tmp_path / "aside").resolve()


def test_derive_package_root_stops_at_the_repo_root(tmp_path, write):
    write(tmp_path / "outer" / "pyproject.toml")
    repo = tmp_path / "outer" / "repo"
    scan = repo / "src"
    scan.mkdir(parents=True)
    assert derive_package_root.derive_package_root(scan, repo) == repo.resolve()
