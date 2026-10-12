"""Everything one seed reaches through a reference extractor — pure."""
from __future__ import annotations

from collections.abc import Callable, Iterable
from pathlib import Path


def closure(seed: str | Path, extract_refs: Callable[[Path], Iterable[str | Path]]) -> list[Path]:
    """`seed` and everything it reaches, breadth-first, each path once."""
    raise NotImplementedError
