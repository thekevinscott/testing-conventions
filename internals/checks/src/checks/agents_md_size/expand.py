"""One instructions file's text with its `@path` imports expanded — pure, the read injected."""
from __future__ import annotations

from checks.agents_md_size.imports import imported_paths

# Claude Code follows an import chain five hops deep; a sixth file is never loaded.
MAX_DEPTH = 5


def expanded(entry: str, read, max_depth: int = MAX_DEPTH) -> str:
    """Everything loading `entry` pulls in, concatenated: the entry first, then imports breadth-first.

    A target `read` reports missing is skipped, and a repeated target terminates a cycle — a file
    two documents both import is loaded once, so it is counted once.
    """
    text = read(entry)
    if text is None:
        return ""
    loaded = {entry: text}
    frontier = [entry]
    for _ in range(max_depth):
        next_frontier = []
        for path in frontier:
            for target in imported_paths(path, loaded[path]):
                if target in loaded:
                    continue
                imported = read(target)
                if imported is None:
                    continue
                loaded[target] = imported
                next_frontier.append(target)
        frontier = next_frontier
    return "".join(loaded.values())
