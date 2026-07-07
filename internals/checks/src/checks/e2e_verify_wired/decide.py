"""The e2e-verify-wired decision — repo-only (#173, #321).

The pure predicate behind the `tc-checks e2e-verify-wired` command: the reusable workflow must
offer the e2e freshness gate (#68) behind an opt-in `run_e2e` input, so both the input
(`run_e2e`) and the command (`e2e verify`) must be present.
"""
from __future__ import annotations

import re


def wires_e2e_verify(text: str) -> bool:
    """True when both the `run_e2e` input and the `e2e verify` command are present."""
    return "run_e2e" in text and re.search(r"e2e verify", text) is not None
