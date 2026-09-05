"""The precedence rule: a default applies only where the consumer set nothing."""
from __future__ import annotations

import os
from pathlib import Path

from .ini_has import ini_has
from .pyproject_has import pyproject_has


def user_set(start, cov_config, key, table):
    """True if the consumer set ``[<table>] <key>`` in any coverage config source at or above
    ``start`` — in which case the plugin leaves the setting alone. ``table`` is coverage's own
    section (``run`` or ``report``), spelled ``[coverage:run]`` / ``[tool.coverage.run]`` per source."""
    paths = []
    if cov_config:
        paths.append(
            cov_config if os.path.isabs(cov_config) else os.path.join(str(start), cov_config)
        )
    base = Path(os.path.abspath(str(start)))
    for directory in (base, *base.parents):
        for name in (".coveragerc", "setup.cfg", "tox.ini", "pyproject.toml"):
            paths.append(str(directory / name))
    for path in paths:
        if not os.path.isfile(path):
            continue
        name = os.path.basename(path)
        if name.endswith(".toml"):
            if pyproject_has(path, table, key):
                return True
        elif name in ("setup.cfg", "tox.ini"):
            if ini_has(path, ["coverage:" + table], key):
                return True
        elif ini_has(path, [table], key):
            return True
    return False
