"""The forward-only tag decision."""
from __future__ import annotations


def decide(*, exists: bool, sha_behind_or_at_tag: bool) -> str:
    """The action two git facts call for: "bootstrap" with no tag yet, "noop" where the
    sha sits at or behind the tag (an out-of-order or re-run release), else "advance"."""
    if not exists:
        return "bootstrap"
    if sha_behind_or_at_tag:
        return "noop"
    return "advance"
