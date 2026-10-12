"""The guard keeping a scope intersection from comparing paths anchored two ways — pure."""
from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path


def one_frame(paths: Sequence[Path]) -> Sequence[Path]:
    """`paths` unchanged, when every one of them is anchored alike."""
    raise NotImplementedError
