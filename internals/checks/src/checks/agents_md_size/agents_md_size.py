"""Whether one instructions file is over its budget — pure.

The check's whole claim, in one comparison: an instructions file is too large when its content,
with every `@path` import already resolved in, runs longer than the budget. Resolution happens in
`resolve_agents_md`; the budget is the caller's.
"""
from __future__ import annotations


def agents_md_size(content: str, max_len: int) -> bool:
    """True when `content` is longer than `max_len` characters."""
    return len(content) > max_len
