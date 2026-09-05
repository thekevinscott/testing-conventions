"""Shared fixtures for the detect unit tier."""
import os
from pathlib import Path

import pytest


@pytest.fixture
def in_dir():
    """Enter a directory for the duration of a test, restoring the original on the way out."""
    original = Path.cwd()
    yield os.chdir
    os.chdir(original)


@pytest.fixture
def write():
    """A helper that creates a path (and its parents) holding the given text."""
    def _write(path: Path, text: str = "") -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    return _write
