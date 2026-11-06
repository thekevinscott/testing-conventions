from pathlib import Path

_SOURCE_GLOBS: dict[str, tuple[str, ...]] = {
    "python": ("*.py",),
    "typescript": ("*.ts", "*.tsx", "*.mts", "*.cts"),
}


def any_match(root: Path, globs: tuple[str, ...]) -> bool:
    """True if any file matching one of `globs` exists anywhere under `root`."""
    for glob in globs:
        for _ in root.rglob(glob):
            return True
    return False


def any_source_match(root: Path, globs: tuple[str, ...]) -> bool:
    """`any_match`, skipping the suite tiers in a `tests/` directory directly beneath `root`."""
    tiers = root / "tests"
    for glob in globs:
        for path in root.rglob(glob):
            if not path.is_relative_to(tiers):
                return True
    return False


def has_source(root: Path, language: str) -> bool:
    """True if `root` holds any source file for `language` (python / typescript)."""
    return any_source_match(root, _SOURCE_GLOBS[language])
