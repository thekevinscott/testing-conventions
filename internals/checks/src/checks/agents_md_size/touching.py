"""The closures a set of changed paths reaches — pure."""
from __future__ import annotations


def touching(closures, changed) -> list:
    """Every (entry, members) closure holding at least one of `changed`."""
    reached = set(changed)
    return [
        (entry, members)
        for entry, members in closures
        if not reached.isdisjoint(path for path, _ in members)
    ]
