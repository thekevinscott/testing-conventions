"""Where one authored list and one factual surface disagree — pure."""

from __future__ import annotations

from collections.abc import Iterable


def surface_drift(surface: str, claimed: Iterable[str], actual: Iterable[str]) -> list[str]:
    """Every disagreement between `claimed` and `actual`, naming the side each name is missing from."""
    raise NotImplementedError
