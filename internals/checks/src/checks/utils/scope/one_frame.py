"""The guard keeping a scope intersection from comparing paths anchored two ways — pure."""
from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path


def one_frame(paths: Sequence[Path]) -> Sequence[Path]:
    """`paths` unchanged, when every one of them is anchored alike.

    An intersection taken across an absolute and a repo-relative path is empty, and an empty scope
    spawns no jobs, so a mixed frame reads as "nothing to do" and silently runs no check.
    """
    if len({path.is_absolute() for path in paths}) > 1:
        raise ValueError(
            "a scope intersection compares paths in one frame: make every seed, reference and "
            "changed path absolute, or every one of them repo-relative"
        )
    return paths
