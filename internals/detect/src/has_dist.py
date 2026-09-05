from pathlib import Path

from file_presence import any_match

_DIST_GLOBS: tuple[str, ...] = ("*.whl", "*.tar.gz", "*.tgz", "*.crate")


def has_dist(root: Path) -> bool:
    """True if a conventional `dist/` under `root` holds a recognized built distribution."""
    dist = root / "dist"
    return dist.is_dir() and any_match(dist, _DIST_GLOBS)
