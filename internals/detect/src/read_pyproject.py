import tomllib
from pathlib import Path


def read_pyproject(root: Path) -> dict:
    """The parsed `pyproject.toml` at `root`, or `{}` if absent or unparseable."""
    manifest = root / "pyproject.toml"
    if not manifest.is_file():
        return {}
    try:
        return tomllib.loads(manifest.read_text())
    except tomllib.TOMLDecodeError:
        return {}
