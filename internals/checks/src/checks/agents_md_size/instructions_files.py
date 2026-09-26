"""Which tracked paths an agent loads as instructions — repo-only.

Discovery is the gate's own decision rather than the caller's: the budget applies to every
instructions file a checkout carries, and a list the author maintains by hand is a list that
misses the next one.
"""
from __future__ import annotations

NAMES = ("AGENTS.md", "CLAUDE.md")


def instructions_files(paths) -> list[str]:
    """Every path in `paths` an agent loads as instructions, sorted, at any depth."""
    raise NotImplementedError
