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


def has_rust_crate(root: Path, package_root: Path) -> bool:
    """True if `root` holds Rust sources belonging to a crate: at least one `.rs` file under
    `root`, and a `Cargo.toml` at `package_root` or somewhere under `root` itself.

    A manifest whose sources are generated at build time has nothing to measure. The
    package-root half is what lets a scan pointed at `src/` — the shape every other language's
    job uses — still find the crate it belongs to."""
    manifest = (package_root / "Cargo.toml").is_file() or any_match(root, ("Cargo.toml",))
    return manifest and any_match(root, ("*.rs",))
