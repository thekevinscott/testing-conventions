"""The added-paths git read — repo-only.

Shells out for exactly one piece of data and returns lines, so the decisions and the orchestration
in `gate.py` never touch a subprocess. The subprocess boundary is injected as `runner`, the seam
the colocated tests drive; `changed_files.py` and `commit_messages.py` hold the sibling reads.
"""
from __future__ import annotations

import subprocess


def added_files(base_sha: str, head_sha: str, runner=subprocess.run) -> list[str]:
    """The paths this branch's own commits added, over the merge-base range."""
    # A fragment satisfies the gate only when the PR adds it, so the diff is filtered to additions.
    result = runner(
        ["git", "diff", "--name-only", "--diff-filter=A", f"{base_sha}...{head_sha}"],
        capture_output=True,
        text=True,
        check=True,
    )
    return [line for line in result.stdout.splitlines() if line]
