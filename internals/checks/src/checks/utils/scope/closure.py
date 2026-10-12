"""Everything one seed reaches through a reference extractor — pure."""
from __future__ import annotations

from collections.abc import Callable, Iterable
from pathlib import Path


def closure(seed: str | Path, extract_refs: Callable[[Path], Iterable[str | Path]]) -> list[Path]:
    """`seed` and everything it reaches, breadth-first, each path once.

    A repeat terminates a cycle, and `extract_refs` is asked about each path once, so a reference
    two paths share is walked once and the depth is bounded only by the references themselves.
    """
    reached: dict[Path, None] = {}
    frontier = [Path(seed)]
    while frontier:
        next_frontier: list[Path] = []
        for candidate in frontier:
            if candidate in reached:
                continue
            reached[candidate] = None
            next_frontier.extend(Path(ref) for ref in extract_refs(candidate))
        frontier = next_frontier
    return list(reached)
