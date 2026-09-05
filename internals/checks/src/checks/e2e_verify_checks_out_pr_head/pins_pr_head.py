"""The block-scoped PR-head-pin decision: only the `e2e-verify` job's own lines count."""
from __future__ import annotations

import re

from checks.e2e_verify_checks_out_pr_head.block import extract_block


def pins_pr_head(text: str) -> bool:
    """True when the `e2e-verify` job's own block pins the PR head commit."""
    block = extract_block(text, re.compile(r"^  e2e-verify:"), re.compile(r"^  packaging:"))
    return "github.event.pull_request.head.sha" in block
