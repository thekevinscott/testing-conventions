from pathlib import Path

from file_presence import any_match


def has_rust_crate(root: Path, package_root: Path) -> bool:
    """True if `root` holds Rust sources belonging to a crate: at least one `.rs` file under
    `root`, and a `Cargo.toml` at `package_root` or somewhere under `root` itself.

    A manifest whose sources are generated at build time has nothing to measure. The
    package-root half is what lets a scan pointed at `src/` — the shape every other language's
    job uses — still find the crate it belongs to."""
    manifest = (package_root / "Cargo.toml").is_file() or any_match(root, ("Cargo.toml",))
    return manifest and any_match(root, ("*.rs",))
