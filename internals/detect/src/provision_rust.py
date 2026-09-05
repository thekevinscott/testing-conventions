from pathlib import Path

from manifest import has_lockfile
from read_package_json import read_package_json
from read_pyproject import read_pyproject


def provision_rust(package_root: Path) -> str:
    """`"true"` when `package_root`'s own manifest declares a Rust-compiling build — a
    `Cargo.toml`, a maturin build backend, or a napi binding — so the suite jobs can provision
    cargo; `"false"` otherwise."""
    if has_lockfile(package_root, "Cargo.toml"):
        return "true"
    backend = read_pyproject(package_root).get("build-system", {}).get("build-backend", "")
    if backend.startswith("maturin"):
        return "true"
    package = read_package_json(package_root)
    if "napi" in package:
        return "true"
    if "@napi-rs/cli" in package.get("devDependencies", {}):
        return "true"
    return "false"
