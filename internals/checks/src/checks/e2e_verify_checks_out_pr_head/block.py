"""The e2e-verify-checks-out-pr-head decision — repo-only (#321).

The pure logic behind the `tc-checks e2e-verify-checks-out-pr-head` command: extract the
`e2e-verify` job's own YAML block and confirm its checkout pins the PR head commit. The check is
block-scoped on purpose — a pin anywhere else in the file (a sibling job) must not satisfy it.
"""
from __future__ import annotations

import re


def extract_block(text: str, start: re.Pattern[str], end: re.Pattern[str]) -> str:
    """The lines from the first `start` match through the first following `end` match.

    Inclusive of both boundary lines, mirroring `awk '/start/,/end/'`. If `end` is never seen the
    block runs to end-of-text; if `start` is never seen the block is empty.
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


def pins_pr_head(text: str) -> bool:
    """True when the `e2e-verify` job's own block pins the PR head commit."""
    block = extract_block(text, re.compile(r"^  e2e-verify:"), re.compile(r"^  packaging:"))
    return "github.event.pull_request.head.sha" in block
