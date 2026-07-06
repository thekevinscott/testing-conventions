"""The rolling-release-wired decisions — repo-only (#235, #321).

The two pure invariants behind the `tc-checks rolling-release-wired` command. `@v0` is a moving
tag, but the reusable workflow runs the *published* npm binary while its own file is frozen at the
tag — so the tag must advance only AFTER that binary publishes, or a consumer gets new-workflow +
old-binary (the #55 `unrecognized subcommand` stranding):

  * `check_move_major_tag` — a dedicated `move-major-tag.yml` exists, is publish-gated
    (`workflow_run` on Release) and concurrency-guarded; and
  * `check_release_no_inline_tag_move` — `release.yml` no longer moves the tag inline
    (`git tag -f v0`), which would re-couple the move to the publish pipeline and drop the gate.
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


def check_release_no_inline_tag_move(text: str) -> Optional[str]:
    """Return an error if `release.yml` still moves the major tag inline (`git tag -f v0`)."""
    if "tag -f v0" in text:
        return (
            "release.yml moves @v0 inline (`git tag -f v0`) — the move belongs in the gated "
            "move-major-tag.yml, not the publish workflow (#235)"
        )
    return None
