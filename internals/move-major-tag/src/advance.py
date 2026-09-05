"""The tag-advance orchestration, over the git boundary and the decision."""
from __future__ import annotations

from decide import decide
from git_ops import fetch_tags, is_ancestor, move_tag, push_tag, tag_exists

WRITING_ACTIONS = frozenset({"bootstrap", "advance"})


def advance(tag: str, sha: str, *, push: bool = True) -> str:
    """Forward-only advance of `tag` to `sha`, returning the action taken. The ancestry check
    runs only when the tag exists: the first run has nothing to compare against."""
    fetch_tags()
    exists = tag_exists(tag)
    sha_behind_or_at_tag = exists and is_ancestor(sha, tag)
    action = decide(exists=exists, sha_behind_or_at_tag=sha_behind_or_at_tag)
    if action in WRITING_ACTIONS:
        move_tag(tag, sha)
        if push:
            push_tag(tag)
    return action
