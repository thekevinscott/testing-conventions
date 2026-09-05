from pathlib import Path

from manifest import has_manifest


def derive_package_root(scan_root: Path, repo_root: Path) -> Path:
    """The package root: the nearest directory at-or-above `scan_root`, down to `repo_root`
    inclusive, holding a manifest; `repo_root` when none is found."""
    scan_root = scan_root.resolve()
    repo_root = repo_root.resolve()
    candidates = []
    for ancestor in [scan_root, *scan_root.parents]:
        candidates.append(ancestor)
        if ancestor == repo_root:
            break
    else:
        candidates.append(repo_root)
    for candidate in candidates:
        if has_manifest(candidate):
            return candidate
    return repo_root
