from pathlib import Path

_MANIFESTS: tuple[str, ...] = ("package.json", "pyproject.toml", "Cargo.toml")


def has_manifest(root: Path) -> bool:
    """True if a package manifest (package.json / pyproject.toml / Cargo.toml) sits at `root`."""
    return any((root / name).is_file() for name in _MANIFESTS)


def has_lockfile(root: Path, name: str) -> bool:
    """True if a file named `name` sits directly at `root`."""
    return (root / name).is_file()
