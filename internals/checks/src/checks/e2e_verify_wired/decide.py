"""The e2e-verify-wired decision: the freshness gate is offered behind the opt-in `run_e2e` input."""
from __future__ import annotations

import re


def wires_e2e_verify(text: str) -> bool:
    """True when both the `run_e2e` input and the `e2e verify` command are present."""
    return "run_e2e" in text and re.search(r"e2e verify", text) is not None
