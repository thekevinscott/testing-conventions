from pathlib import Path
from typing import Optional

from read_cargo import read_cargo


def cargo_workspace_root(package_root: Path, repo_root: Path) -> Optional[Path]:
    """The nearest strict ancestor of `package_root` (up to `repo_root` inclusive) whose
    `Cargo.toml` carries a `[workspace]` table, or `None` — cargo resolves the target
    directory there, so the Rust build cache keys on that ancestor's `target/`."""
    package_root = package_root.resolve()
    repo_root = repo_root.resolve()
    if package_root == repo_root:
        return None
    ancestors = []
    for ancestor in package_root.parents:
        ancestors.append(ancestor)
        if ancestor == repo_root:
            break
    else:
        ancestors.append(repo_root)
    for ancestor in ancestors:
        if "workspace" in read_cargo(ancestor):
            return ancestor
    return None


def is_workspace_member(package_root: Path, repo_root: Path) -> bool:
    """True when an ancestor workspace owns `package_root`, so a derived build must redirect
    `--target-dir` back into the package's own tree."""
    return cargo_workspace_root(package_root, repo_root) is not None
