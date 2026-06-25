#!/usr/bin/env python3
"""Advance the moving major tag `@v0` to a released commit — forward-only (#235).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it is invoked by the `Move major tag` workflow
(.github/workflows/move-major-tag.yml) after a successful Release, to force-move the moving
major tag forward so every `@v0` consumer tracks `main` on its next run. Keeping the logic
here — rather than inline in the workflow YAML — lets it carry real integration and e2e tests
(see `tests/`).

git is the one external dependency. It lives behind the small boundary functions below
(`fetch_tags` / `tag_exists` / `is_ancestor` / `move_tag` / `push_tag`) so an *integration*
test can mock them and exercise the real `advance` orchestration, while an *e2e* test runs the
whole thing against a real git repo wired to a local remote.

Inputs come from the environment (set by the workflow):
  SHA   the released commit to advance the tag to (required).
  TAG   the moving major tag to advance (default 'v0').
"""
from __future__ import annotations

import os
import subprocess


def _git(*args: str) -> subprocess.CompletedProcess[str]:
    """Run a git command, capturing output. Does not raise on non-zero — callers inspect."""
    return subprocess.run(["git", *args], capture_output=True, text=True)


# --- git boundary (the external dependency an integration test mocks) ---

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


# --- orchestration (runs for real under both test kinds; only git is mocked) ---

def decide(*, exists: bool, sha_behind_or_at_tag: bool) -> str:
    """The forward-only decision, as a pure function of two git facts.

    - no tag yet                       -> "bootstrap" (the first run creates it)
    - tag exists, sha at or behind it  -> "noop" (never rewind: an out-of-order / re-run release)
    - tag exists, sha ahead of it      -> "advance"
    """
    if not exists:
        return "bootstrap"
    if sha_behind_or_at_tag:
        return "noop"
    return "advance"


def advance(tag: str, sha: str, *, push: bool = True) -> str:
    """Forward-only advance of `tag` to `sha`. Returns the action taken.

    Fetches tags, decides via `decide`, then — unless the decision is "noop" — moves the tag
    and (when `push`) force-pushes it. The ancestry check is skipped when the tag is absent:
    there is nothing to compare against, and the first run simply bootstraps the tag.
    """
    fetch_tags()
    exists = tag_exists(tag)
    sha_behind_or_at_tag = exists and is_ancestor(sha, tag)
    action = decide(exists=exists, sha_behind_or_at_tag=sha_behind_or_at_tag)
    if action == "noop":
        return action
    move_tag(tag, sha)
    if push:
        push_tag(tag)
    return action


def main() -> int:
    sha = os.environ.get("SHA", "").strip()
    if not sha:
        print("::error::SHA is required (the released commit to advance the tag to)")
        return 1
    tag = os.environ.get("TAG", "v0").strip() or "v0"
    action = advance(tag, sha)
    print({
        "bootstrap": f"{tag} did not exist yet; bootstrapped it at {sha}",
        "advance": f"advanced {tag} -> {sha}",
        "noop": f"{tag} is already at or ahead of {sha}; nothing to do",
    }[action])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
