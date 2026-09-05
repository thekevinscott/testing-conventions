"""The release-side invariant: `release.yml` moves no tag inline — the gated workflow does."""
from __future__ import annotations

from typing import Optional


def check_release_no_inline_tag_move(text: str) -> Optional[str]:
    """Return an error if `release.yml` still moves the major tag inline (`git tag -f v0`)."""
    if "tag -f v0" in text:
        return (
            "release.yml moves @v0 inline (`git tag -f v0`) — the move belongs in the gated "
            "move-major-tag.yml, not the publish workflow (#235)"
        )
    return None
