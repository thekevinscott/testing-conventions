import tomllib
from pathlib import Path


def read_cargo(package_root: Path) -> dict:
    """The parsed `Cargo.toml` at `package_root`, or `{}` if absent or unparseable — the
    `OSError` catch covers a missing file, so there is no separate presence guard."""
    try:
        return tomllib.loads((package_root / "Cargo.toml").read_text())
    except (OSError, tomllib.TOMLDecodeError):
        return {}
