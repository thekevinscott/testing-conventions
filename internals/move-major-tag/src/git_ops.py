"""The git boundary the tag-advance helper runs against."""
from __future__ import annotations

import subprocess


def _git(*args: str) -> subprocess.CompletedProcess[str]:
    """Run a git command, capturing output. Does not raise on non-zero — callers inspect."""
    return subprocess.run(["git", *args], capture_output=True, text=True)


def fetch_tags() -> None:
    """Sync local tags with the remote, so the ancestry check sees the current `@v0`."""
    _git("fetch", "--force", "--tags", "origin").check_returncode()


def tag_exists(tag: str) -> bool:
    """True if `tag` resolves to an object in this repo."""
    return _git("rev-parse", "-q", "--verify", f"refs/tags/{tag}").returncode == 0


def is_ancestor(maybe_ancestor: str, descendant: str) -> bool:
    """True if `maybe_ancestor` is an ancestor of (or identical to) `descendant`."""
    return (
        _git("merge-base", "--is-ancestor", maybe_ancestor, descendant).returncode == 0
    )


def move_tag(tag: str, sha: str) -> None:
    """Force-move the lightweight tag `tag` onto `sha`, locally."""
    _git("tag", "-f", tag, sha).check_returncode()


def push_tag(tag: str) -> None:
    """Force-push `tag` to origin."""
    _git("push", "-f", "origin", f"refs/tags/{tag}").check_returncode()
