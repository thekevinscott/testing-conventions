"""The tracked-paths git read — repo-only.

Shells out for exactly one piece of data — every path a checkout materializes under `root` — so
the decision layer never touches a subprocess. The subprocess boundary is injected as `runner`,
the seam the colocated tests drive.
"""
from __future__ import annotations

import subprocess


def tracked_paths(root: str, runner=subprocess.run) -> list[str]:
    """Every path git tracks under `root`, relative to `root`."""
    result = runner(
        ["git", "ls-files"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    )
    return [line for line in result.stdout.splitlines() if line]
