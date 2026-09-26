"""The changed-paths git read — repo-only. The subprocess boundary is injected as `runner`."""
from __future__ import annotations

import subprocess


def changed_paths(root: str, base: str, runner=subprocess.run) -> list[str]:
    """Every path `root` changed since it forked from `base`, relative to `root`."""
    result = runner(
        ["git", "diff", "--name-only", f"{base}...HEAD"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    )
    return [line for line in result.stdout.splitlines() if line]
