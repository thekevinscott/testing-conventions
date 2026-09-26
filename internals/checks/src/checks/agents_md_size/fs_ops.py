"""The instructions-file read — repo-only.

Reads exactly one piece of data — a file's text — so the decision layer never touches the
filesystem. The filesystem boundary is injected as `opener`, the seam the colocated tests drive.
A missing target reads as None; an empty file reads as ''.
"""
from __future__ import annotations

from pathlib import Path
from typing import Optional


def read_text(root: str, path: str, opener=Path) -> Optional[str]:
    """`path`'s text under `root`, or None when the checkout holds no readable file there."""
    try:
        return opener(root, path).read_text(encoding="utf-8")
    except OSError:
        return None
