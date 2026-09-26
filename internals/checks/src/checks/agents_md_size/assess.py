"""The size verdict for one instructions-file closure — pure."""
from __future__ import annotations

from checks.agents_md_size.limits import Limits


def assess(members, limits: Limits) -> tuple[int, int, str]:
    """(lines, bytes, level) for the closure: `fail` past the hard budget, `warn` past the soft."""
    lines = sum(len(text.splitlines()) for _, text in members)
    byte_count = sum(len(text.encode("utf-8")) for _, text in members)
    if lines > limits.max_lines or byte_count > limits.max_bytes:
        return lines, byte_count, "fail"
    if lines > limits.warn_lines or byte_count > limits.warn_bytes:
        return lines, byte_count, "warn"
    return lines, byte_count, "ok"
