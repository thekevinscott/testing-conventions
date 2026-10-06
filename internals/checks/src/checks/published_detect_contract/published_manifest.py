"""Read the detect action manifest as a published ref carries it."""
from __future__ import annotations

import subprocess

from checks.utils.verify_release.run_text import run_text

DETECT_MANIFEST = ".github/actions/detect/action.yml"


def published_manifest(ref: str, run=subprocess.run) -> str:
    """The detect action manifest at `ref` — the bytes GitHub resolves for a consumer there."""
    return run_text(run, ["git", "show", f"{ref}:{DETECT_MANIFEST}"])
