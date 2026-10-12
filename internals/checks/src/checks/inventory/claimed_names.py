"""The names an inventory claims under one field, flattened — pure."""

from __future__ import annotations

from collections.abc import Iterable, Mapping


def claimed_names(entries: Iterable[Mapping[str, object]], field: str) -> list[str]:
    """Every value `entries` carry at `field`, a list-valued field flattened into its members."""
    raise NotImplementedError
