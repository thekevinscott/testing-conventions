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


def has_source(root: Path, language: str) -> bool:
    """True if `root` holds any source file for `language` (python / typescript)."""
    return any_match(root, _SOURCE_GLOBS[language])


def has_rust_crate(root: Path) -> bool:
    """True if `root` holds a Rust crate to check: a `Cargo.toml` and at least one `.rs` file —
    a manifest whose sources are generated at build time has nothing to measure."""
    return any_match(root, ("Cargo.toml",)) and any_match(root, ("*.rs",))
