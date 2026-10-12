"""The names an inventory claims under one field, flattened — pure."""

from __future__ import annotations

from collections.abc import Iterable, Mapping


def claimed_names(entries: Iterable[Mapping[str, object]], field: str) -> list[str]:
    """Every value `entries` carry at `field`, a list-valued field flattened into its members.

    A check runs under one command and under any number of jobs, so one accessor reads a scalar and
    a list without the caller knowing which it asked for. An entry missing the field claims nothing,
    which is how a job that runs no check claims no command.
    """
    names: list[str] = []
    for entry in entries:
        claimed = entry.get(field)
        if isinstance(claimed, list):
            names.extend(str(member) for member in claimed)
        elif claimed is not None:
            names.append(str(claimed))
    return names
