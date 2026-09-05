"""Reading a coverage key out of a TOML config source."""
from __future__ import annotations

from . import tomlcompat


def pyproject_has(path, table, key):
    """True if ``[tool.coverage.<table>]`` in the TOML file at ``path`` sets ``key``."""
    try:
        with open(path, "rb") as handle:
            data = tomlcompat.load(handle)
    except Exception:
        return False
    return key in data.get("tool", {}).get("coverage", {}).get(table, {})
