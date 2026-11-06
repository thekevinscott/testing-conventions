from pathlib import Path


def any_source_match(root: Path, globs: tuple[str, ...]) -> bool:
    """True if a file matching one of `globs` exists under `root`, outside `root/tests/`."""
    tiers = root / "tests"
    for glob in globs:
        for path in root.rglob(glob):
            if not path.is_relative_to(tiers):
                return True
    return False
