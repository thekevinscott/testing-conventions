"""Detecting whether the consumer has set a coverage option themselves.

Pure functions — the precedence rule ("our default applies only where the
consumer set nothing") lives here, separate from the pytest hook glue, so it can
be unit-tested directly.
"""
from __future__ import annotations

import configparser
import os
from pathlib import Path

from .tomlcompat import load as _load_toml


def _ini_has(path, sections, key):
    parser = configparser.ConfigParser()
    try:
        parser.read(path)
    except Exception:
        return False
    return any(parser.has_section(s) and parser.has_option(s, key) for s in sections)


def _pyproject_has(path, table, key):
    try:
        with open(path, "rb") as handle:
            data = _load_toml(handle)
    except Exception:
        return False
    return key in data.get("tool", {}).get("coverage", {}).get(table, {})


def user_set(start, cov_config, key, table):
    """True if the consumer set ``[<table>] <key>`` in any coverage config source
    at or above ``start`` — in which case the plugin leaves the setting alone.

    ``table`` is the coverage section (``run`` or ``report``); it maps to ``[run]``
    in a ``.coveragerc`` or a custom ``--cov-config``, ``[coverage:run]`` in the
    shared ``setup.cfg`` / ``tox.ini``, and ``[tool.coverage.run]`` in a ``.toml``.
    """
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
            if _pyproject_has(path, table, key):
                return True
        elif name in ("setup.cfg", "tox.ini"):
            if _ini_has(path, ["coverage:" + table], key):
                return True
        elif _ini_has(path, [table], key):
            return True
    return False
