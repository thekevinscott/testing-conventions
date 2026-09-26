"""One instructions file's content, with every `@path` import resolved in."""
# No `from __future__ import annotations`: it would make `str | Path` an unevaluated string, and the
# mutation gate then reports every operator swap inside it as an unkillable survivor.
from pathlib import Path

from checks.agents_md_size.resolve_agents_md.imported_paths import imported_paths

# Claude Code follows an import chain five hops deep; a sixth file is never loaded.
MAX_DEPTH = 5


def resolve_agents_md(path_to_agents_md: str | Path) -> str:
    """Everything loading `path_to_agents_md` pulls in: the entry first, then imports breadth-first.

    An unreadable target is skipped, and a repeated target terminates a cycle — a file two documents
    both import is loaded once, so it is counted once.
    """
    loaded: dict[Path, str] = {}
    frontier = [Path(path_to_agents_md).expanduser().resolve()]
    for _ in range(MAX_DEPTH + 1):
        next_frontier: list[Path] = []
        for candidate in frontier:
            if candidate in loaded:
                continue
            try:
                text = candidate.read_text(encoding="utf-8")
            except OSError:
                continue
            loaded[candidate] = text
            next_frontier.extend(imported_paths(candidate, text))
        frontier = next_frontier
    return "".join(loaded.values())
