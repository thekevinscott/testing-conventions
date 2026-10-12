"""The seeds a change reaches: `closure ∩ changed`, a check's whole scoping decision — pure."""
from __future__ import annotations

from collections.abc import Callable, Iterable
from pathlib import Path

from checks.utils.scope.closure import closure
from checks.utils.scope.one_frame import one_frame


def seeds_in_scope(
    seeds: Iterable[str | Path],
    extract_refs: Callable[[Path], Iterable[str | Path]],
    changed: Iterable[str | Path],
) -> list[Path]:
    """Each seed whose closure holds a changed path, in the order `seeds` gave them."""
    changed_paths = [Path(path) for path in changed]
    walked = [(Path(seed), closure(seed, extract_refs)) for seed in seeds]
    one_frame([*changed_paths, *(member for (_, reached) in walked for member in reached)])
    touched = set(changed_paths)
    return [seed for (seed, reached) in walked if not touched.isdisjoint(reached)]
