"""Reading a coverage key out of an ini-style config source."""
from __future__ import annotations

import configparser


def ini_has(path, sections, key):
    """True if any of ``sections`` in the ini file at ``path`` sets ``key``."""
    parser = configparser.ConfigParser()
    try:
        parser.read(path)
    except Exception:
        return False
    return any(parser.has_section(s) and parser.has_option(s, key) for s in sections)
