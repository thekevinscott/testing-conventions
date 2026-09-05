from pathlib import Path


def primary_language(package_root: Path) -> str:
    """The package's primary language by manifest, or `''`: a `pyproject.toml` is `python`, else
    a `package.json` is `typescript`, else a `Cargo.toml` is `rust` — the priority reads a
    binding's second manifest as the private core, not the published artifact."""
    if (package_root / "pyproject.toml").is_file():
        return "python"
    if (package_root / "package.json").is_file():
        return "typescript"
    if (package_root / "Cargo.toml").is_file():
        return "rust"
    return ""
