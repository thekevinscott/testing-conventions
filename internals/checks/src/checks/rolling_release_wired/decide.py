"""The move-major-tag workflow invariant: `@v0` advances only after the binary publishes.

The reusable workflow runs the *published* npm binary while its own file is frozen at the tag, so
an early tag move hands a consumer new-workflow + old-binary. The release-side half of the
invariant lives in `check_release_no_inline_tag_move.py`.
"""
from __future__ import annotations

from typing import Optional


def check_move_major_tag(text: Optional[str]) -> Optional[str]:
    """Validate the dedicated move-major-tag workflow.

    `text` is the file's contents, or `None` when the file is absent. Returns the first failing
    reason (missing file / not publish-gated / not concurrency-guarded), else `None`.
    """
    if text is None:
        return (
            "missing .github/workflows/move-major-tag.yml — the major tag has no dedicated "
            "advance workflow (#235)"
        )
    if "workflow_run:" not in text:
        return (
            ".github/workflows/move-major-tag.yml is not gated on a workflow_run (Release "
            "completion) — @v0 could advance before the binary publishes (#235)"
        )
    if "concurrency:" not in text:
        return (
            ".github/workflows/move-major-tag.yml has no concurrency group — out-of-order tag "
            "moves are unguarded (#235)"
        )
    return None
