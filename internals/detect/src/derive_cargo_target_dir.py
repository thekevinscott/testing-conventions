from pathlib import Path
from typing import Optional


def derive_cargo_target_dir(package_root_rel: Path, workspace_root_rel: Optional[Path]) -> str:
    """The repo-root-relative `target/` the cache steps key on: the workspace root's when the
    package is a member, else the package root's own."""
    root = workspace_root_rel if workspace_root_rel is not None else package_root_rel
    return f"{root}/target"
