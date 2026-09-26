"""Whether one instructions file fits its budget."""
from __future__ import annotations

from pathlib import Path

from checks.agents_md_size.resolve_agents_md.resolve_agents_md import resolve_agents_md


def agents_md_size(path_to_agents_md: str | Path, max_len: int) -> bool:
    """True when the file at `path_to_agents_md`, imports resolved in, is under `max_len` characters."""
    contents = resolve_agents_md(path_to_agents_md)
    return len(contents) < max_len
