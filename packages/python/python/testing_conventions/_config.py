"""Detecting whether the consumer has set a coverage option themselves.

Pure functions, separated from the pytest hook glue (``_pytest_plugin``) so the
precedence logic — "our default applies only where the consumer set nothing" —
is unit-testable on its own.
"""
from __future__ import annotations

import configparser
import os

try:
    import tomllib
except ImportError:  # pragma: no cover - Python 3.9 / 3.10 fallback; coverage ships tomli there
    try:
        import tomli as tomllib
    except ImportError:
        tomllib = None


def _ini_has(path, sections, key):
    parser = configparser.ConfigParser()
    try:
        parser.read(path)
    except Exception:  # pragma: no cover - unreadable ini is treated as "unset"
        return False
    return any(parser.has_section(s) and parser.has_option(s, key) for s in sections)


def _pyproject_has(path, table, key):
    if tomllib is None:  # pragma: no cover - tomllib present on supported runners
        return False
    try:
        with open(path, "rb") as handle:
            data = tomllib.load(handle)
    except Exception:  # pragma: no cover - malformed toml is treated as "unset"
        return False
    return key in data.get("tool", {}).get("coverage", {}).get(table, {})


def user_set(start, cov_config, key, table):
    """True if the consumer set ``[<table>] <key>`` in any coverage config source
    at or above ``start`` — in which case the plugin leaves the setting alone.

    ``table`` is the coverage section name (``run`` or ``report``); it maps to
    ``[run]`` in a ``.coveragerc``, ``[coverage:run]`` in ``setup.cfg`` / ``tox.ini``,
    and ``[tool.coverage.run]`` in ``pyproject.toml``.
    """
    paths = []
    if cov_config:
        paths.append(
            cov_config if os.path.isabs(cov_config) else os.path.join(str(start), cov_config)
        )
    directory = os.path.abspath(str(start))
    while True:
        for name in (".coveragerc", "setup.cfg", "tox.ini", "pyproject.toml"):
            paths.append(os.path.join(directory, name))
        parent = os.path.dirname(directory)
        if parent == directory:
            break
        directory = parent
    for path in paths:
        if not os.path.isfile(path):
            continue
        name = os.path.basename(path)
        if name.endswith(".toml"):
            # pyproject.toml or a `.toml` passed as --cov-config: [tool.coverage.<table>]
            if _pyproject_has(path, table, key):
                return True
        elif name in ("setup.cfg", "tox.ini"):
            # coverage prefixes its sections in these shared files
            if _ini_has(path, ["coverage:" + table], key):
                return True
        elif _ini_has(path, [table], key):
            # .coveragerc, or a custom --cov-config: bare [run] / [report]
            return True
    return False
