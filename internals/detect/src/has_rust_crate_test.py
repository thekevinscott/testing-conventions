import has_rust_crate


def test_has_rust_crate_needs_a_manifest_and_real_source(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "src" / "lib.rs", "pub fn f() {}\n")
    assert has_rust_crate.has_rust_crate(tmp_path, tmp_path) is True


def test_has_rust_crate_is_false_for_a_manifest_with_no_source(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    assert has_rust_crate.has_rust_crate(tmp_path, tmp_path) is False


def test_has_rust_crate_is_false_for_source_with_no_manifest(tmp_path, write):
    write(tmp_path / "src" / "lib.rs", "pub fn f() {}\n")
    assert has_rust_crate.has_rust_crate(tmp_path / "src", tmp_path / "src") is False


def test_has_rust_crate_finds_the_manifest_above_a_src_scan(tmp_path, write):
    write(tmp_path / "Cargo.toml", '[package]\nname = "c"\n')
    write(tmp_path / "src" / "lib.rs", "pub fn f() {}\n")
    assert has_rust_crate.has_rust_crate(tmp_path / "src", tmp_path) is True


def test_has_rust_crate_is_false_when_the_package_root_is_another_languages(tmp_path, write):
    write(tmp_path / "package.json", "{}\n")
    write(tmp_path / "src" / "build.rs", "fn main() {}\n")
    assert has_rust_crate.has_rust_crate(tmp_path / "src", tmp_path) is False
