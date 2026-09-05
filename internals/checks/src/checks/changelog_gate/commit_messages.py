"""The commit-bodies git read, over the same injected-runner seam as `git_ops.py`."""
from __future__ import annotations

import subprocess


def commit_messages(base_sha: str, head_sha: str, runner=subprocess.run) -> str:
    """The raw bodies of every commit in the range, concatenated."""
    # `%B` is the raw body: the `skip-changelog:` bypass is found on any line, which git's own
    # trailer parser would not do.
    result = runner(
        ["git", "log", "--format=%B", f"{base_sha}..{head_sha}"],
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout
