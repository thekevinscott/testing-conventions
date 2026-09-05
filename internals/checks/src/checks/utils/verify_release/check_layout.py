"""Check the promoted commit's archive carries the `detect`-action fetch targets."""
from __future__ import annotations

import subprocess

from checks.utils.verify_release.run_bytes import run_bytes

# A consumer's `detect` action reaches its implementation through these two paths, so both must
# survive into the promoted commit's archived tree or every consumer's detect job dies at `@v0`.
REQUIRED_ACTION_PATHS = (
    ".github/actions/detect/action.yml",
    "internals/detect/src/detect.py",
)


def missing_paths(present: set[str], required: tuple[str, ...] = REQUIRED_ACTION_PATHS) -> list[str]:
    """The `required` paths absent from `present`, in the given order."""
    return [path for path in required if path not in present]


def check_layout(sha: str, run=subprocess.run) -> list[str]:
    """The required `detect`-action paths absent from `git archive <sha>` — empty when all present."""
    archive = run_bytes(run, ["git", "archive", "--format=tar", sha])
    names = run_bytes(run, ["tar", "--list", "--file", "-"], input=archive).decode()
    return missing_paths({line.rstrip("/") for line in names.splitlines() if line})
