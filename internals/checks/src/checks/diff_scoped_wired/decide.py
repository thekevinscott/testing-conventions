"""Decide whether both diff-scoped `--base` checks — co-change and changed-line coverage — are
invoked, since a missing one silently never runs in CI."""
from __future__ import annotations

import re


def wires_diff_scoped(text: str) -> bool:
    """True when both diff-scoped `--base` checks are invoked."""
    return (
        re.search(r"colocated-test .*--base", text) is not None
        and re.search(r"unit coverage .*--base", text) is not None
    )
