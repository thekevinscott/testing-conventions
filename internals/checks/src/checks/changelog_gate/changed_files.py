"""The changed-paths git read, over the same injected-runner seam as `git_ops.py`."""
from __future__ import annotations

import subprocess


def changed_files(base_sha: str, head_sha: str, runner=subprocess.run) -> list[str]:
    """Every path this branch's own commits changed."""
    # Three-dot: the diff from the merge base, so paths the base branch changed after this branch
    # forked are not read as this PR's work.
    result = runner(
        ["git", "diff", "--name-only", f"{base_sha}...{head_sha}"],
        capture_output=True,
        text=True,
        check=True,
    )
    return [line for line in result.stdout.splitlines() if line]
