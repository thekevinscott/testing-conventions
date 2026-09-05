from pathlib import Path
from typing import Optional

from manifest import has_lockfile
from read_package_json import read_package_json


def _package_manager_from_field(value: str) -> Optional[str]:
    """The manager name from a `packageManager` value like `pnpm@8.6.0`, or `None` when empty."""
    return value.partition("@")[0] if value else None


def ts_package_manager(package_root: Path) -> str:
    """The package manager `package_root` is set up for: the `packageManager` declaration, else
    the manager whose lockfile sits there, else `pnpm`."""
    declared = _package_manager_from_field(read_package_json(package_root).get("packageManager", ""))
    if declared:
        return declared
    if has_lockfile(package_root, "pnpm-lock.yaml"):
        return "pnpm"
    if has_lockfile(package_root, "package-lock.json"):
        return "npm"
    return "pnpm"
