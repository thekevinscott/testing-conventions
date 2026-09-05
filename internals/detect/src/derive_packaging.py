from pathlib import Path

from cargo_workspace import is_workspace_member
from read_cargo import read_cargo
from read_pyproject import read_pyproject
from ts_package_manager import ts_package_manager


def derive_packaging(package_root: Path, primary: str, repo_root: Path) -> str:
    """The command that builds the publishable distribution from the manifest alone — `uv build`,
    `<pm> pack --pack-destination dist`, or `cargo package` — or `''` when the manifest doesn't
    standardize one. `docs/internals/repo.md` carries the per-language derivations."""
    def rust_package() -> str:
        if "package" not in read_cargo(package_root):
            return ""
        if is_workspace_member(package_root, repo_root):
            return "cargo package --target-dir target"
        return "cargo package"

    builders = {
        "python": lambda: "uv build" if "project" in read_pyproject(package_root) else "",
        "typescript": lambda: f"{ts_package_manager(package_root)} pack --pack-destination dist",
        "rust": rust_package,
    }
    build = builders.get(primary)
    return build() if build else ""
