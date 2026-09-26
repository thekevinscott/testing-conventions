"""The always-on instructions files among a set of paths — pure."""
from __future__ import annotations

import posixpath

NAMES = frozenset({"AGENTS.md", "CLAUDE.md"})


def instructions_files(paths) -> list[str]:
    """Every path an agent loads at launch, sorted."""
    return sorted(path for path in paths if posixpath.basename(path) in NAMES)
