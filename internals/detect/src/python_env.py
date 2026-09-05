from pathlib import Path

from read_pyproject import read_pyproject


def python_env(package_root: Path) -> str:
    """`uv` when `package_root`'s `pyproject.toml` declares a `[project]` table, else `pip`."""
    return "uv" if "project" in read_pyproject(package_root) else "pip"
