"""Bound each `steps:` list item to its own lines, so step-local facts (a `CLI_COMMAND` env
line) are checked per step rather than file-wide."""
from __future__ import annotations

import re

STEP_BODY = "      "
STEP_START = re.compile(r"(?m)^(?=      - )")


def step_blocks(text: str) -> list[str]:
    """Every `steps:` list item in `text`, bounded to the lines that belong to it.

    A chunk ends at the next list item (the split point) or at the first line indented shallower
    than a step body — the job-level key that closes the `steps:` list.
    """
    blocks = []
    for chunk in STEP_START.split(text)[1:]:
        lines = chunk.split("\n")
        ends = (i for i, line in enumerate(lines) if line.strip() and not line.startswith(STEP_BODY))
        blocks.append("\n".join(lines[: next(ends, len(lines))]))
    return blocks
