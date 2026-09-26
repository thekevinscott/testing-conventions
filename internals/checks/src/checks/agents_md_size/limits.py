"""The line and byte budgets an instructions-file closure is held to.

See docs/internals/repo.md, "Instructions file size budget", for the derivation.
"""
from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Limits:
    """The soft and hard budgets, in lines and in bytes."""

    warn_lines: int = 200
    warn_bytes: int = 16384
    max_lines: int = 400
    max_bytes: int = 32768
