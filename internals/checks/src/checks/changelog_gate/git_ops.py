"""The changelog-gate git reads — repo-only (#496).

Each function shells out for exactly one piece of data and returns raw text or lines, so the
decisions in `decide.py` and the orchestration in `gate.py` never touch a subprocess. The
subprocess boundary is injected as `runner`, the seam the colocated tests drive.
"""
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


def added_files(base_sha: str, head_sha: str, runner=subprocess.run) -> list[str]:
    """The paths this branch's own commits added, over the same merge-base range."""
    # A fragment satisfies the gate only when the PR adds it, so the diff is filtered to additions.
    result = runner(
        ["git", "diff", "--name-only", "--diff-filter=A", f"{base_sha}...{head_sha}"],
        capture_output=True,
        text=True,
        check=True,
    )
    return [line for line in result.stdout.splitlines() if line]


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
