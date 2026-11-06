"""Find the colocated unit test a mutated module's suite starts from."""
from __future__ import annotations

from pathlib import Path


def colocated_test_path(module_path):
    """The ``<stem>_test.py`` beside ``module_path``, or ``None`` when no such file exists."""
    path = Path(module_path)
    candidate = path.with_name(f"{path.stem}_test.py")
    return candidate if candidate.is_file() else None
