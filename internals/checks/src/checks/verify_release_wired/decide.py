"""Decide whether the layout check and the version-pinned suite run and the tag-move job `needs:`
them, checked inside the move job's own block so a `needs:` edge on another job satisfies none of
it."""
from __future__ import annotations

from typing import Optional

from checks.utils.job_block import iter_job_blocks

LAYOUT_STEP = "tc-checks verify-release check-layout"
SUITE_STEP = "tc-checks verify-release dispatch-and-wait"
MOVE_STEP = "move_major_tag.py"
DISPATCHED_WORKFLOWS = ("testing-conventions-selftest.yml", "dogfood.yml")
VERIFY_JOBS = ("verify-layout", "verify-suite")


def check_move_gated_on_verification(text: Optional[str]) -> Optional[str]:
    """Validate that the tag move is gated on a pre-move verification.

    `text` is `move-major-tag.yml`'s contents, or `None` when the file is absent. Returns the first
    failing reason (missing file / no layout check / no suite dispatch / a suite workflow not
    dispatched / the move job not gated on the verify jobs), else `None`.
    """
    if text is None:
        return (
            "missing .github/workflows/move-major-tag.yml — no workflow advances @v0, so there is "
            "nothing to gate on verification (#357)"
        )
    missing = []
    if LAYOUT_STEP not in text:
        missing.append("the layout check (`tc-checks verify-release check-layout`)")
    if SUITE_STEP not in text:
        missing.append("the version-pinned suite dispatch (`tc-checks verify-release dispatch-and-wait`)")
    for workflow in DISPATCHED_WORKFLOWS:
        if workflow not in text:
            missing.append(f"a verification dispatch of {workflow}")
    if missing:
        return (
            "move-major-tag.yml advances @v0 with no pre-move verification — missing "
            + ", ".join(missing)
            + " — so a release that publishes a binary + workflow combination the consumer surface "
            "rejects would still move @v0 and turn main/consumers red (#357)"
        )
    move_jobs = [(name, block) for name, block in iter_job_blocks(text) if MOVE_STEP in block]
    if not move_jobs:
        return (
            "no job in move-major-tag.yml runs move_major_tag.py — the forward-only tag move must "
            "stay the gated helper, not inline logic (#357, #235)"
        )
    for name, block in move_jobs:
        ungated = [job for job in VERIFY_JOBS if job not in block]
        if ungated:
            return (
                f"the `{name}` job advances @v0 without `needs:` on {', '.join(ungated)} — the move "
                "isn't gated on verification, so a red layout check or a red version-pinned suite "
                "wouldn't stop the promotion (#357)"
            )
    return None
