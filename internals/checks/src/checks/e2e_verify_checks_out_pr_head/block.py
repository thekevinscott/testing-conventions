"""Line-block extraction between two boundary patterns, mirroring `awk '/start/,/end/'`."""
from __future__ import annotations

import re


def extract_block(text: str, start: re.Pattern[str], end: re.Pattern[str]) -> str:
    """The lines from the first `start` match through the first following `end` match.

    Inclusive of both boundary lines. If `end` is never seen the block runs to end-of-text; if
    `start` is never seen the block is empty.
    """
    lines = text.splitlines()
    block: list[str] = []
    in_block = False
    for line in lines:
        if not in_block:
            if start.match(line):
                in_block = True
                block.append(line)
            continue
        block.append(line)
        if end.match(line):
            break
    return "\n".join(block)
