"""The transitive `@path` import closure of one instructions file — pure, with the read injected."""
from __future__ import annotations

from checks.agents_md_size.imports import import_targets
from checks.agents_md_size.resolve_target import resolve_target

# Claude Code follows an import chain five hops deep; a sixth file is never loaded.
MAX_DEPTH = 5


def closure(entry: str, read, max_depth: int = MAX_DEPTH) -> list[tuple[str, str]]:
    """Every (path, text) loading `entry` pulls in, entry first, breadth-first, deduplicated.

    A target that resolves outside the checkout, or that `read` reports missing, is skipped; a
    repeated target terminates a cycle.
    """
    text = read(entry)
    if text is None:
        return []
    members = {entry: text}
    frontier = [entry]
    for _ in range(max_depth):
        next_frontier = []
        for path in frontier:
            for target in import_targets(members[path]):
                resolved = resolve_target(path, target)
                if resolved is None or resolved in members:
                    continue
                imported = read(resolved)
                if imported is None:
                    continue
                members[resolved] = imported
                next_frontier.append(resolved)
        frontier = next_frontier
    return list(members.items())
