"""The detect-routes-python decision — repo-only (#189; epic #302, #305, #324)."""
from __future__ import annotations

import json


def routes_python(isolation_languages: str) -> bool:
    """True when Python is a member of the compact JSON array the detect action emits.

    `isolation_languages` is the JSON array detect emits (e.g. `["python"]`, `["python","rust"]`,
    `[]`). Python routes in when it is a member of that array; a malformed or non-array value does
    not route Python in.
    """
    try:
        languages = json.loads(isolation_languages)
    except (json.JSONDecodeError, ValueError):
        return False
    return isinstance(languages, list) and "python" in languages
