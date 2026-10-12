"""The seeds a change reaches: `closure ∩ changed`, a check's whole scoping decision — pure."""
from __future__ import annotations

from collections.abc import Callable, Iterable
from pathlib import Path


def seeds_in_scope(
    seeds: Iterable[str | Path],
    extract_refs: Callable[[Path], Iterable[str | Path]],
    changed: Iterable[str | Path],
) -> list[Path]:
    """Each seed whose closure holds a changed path, in the order `seeds` gave them."""
    raise NotImplementedError
