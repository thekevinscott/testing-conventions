"""The diff-scoped-wired decision — repo-only (#172, #321).

The pure predicate behind the `tc-checks diff-scoped-wired` command: both diff-scoped `--base`
checks (the commit-scoped co-change check #161 and the changed-line coverage check #162) must be
invoked, or a diff-scoped rule silently never runs in CI.
"""
from __future__ import annotations

import re


def wires_diff_scoped(text: str) -> bool:
    """True when both diff-scoped `--base` checks (co-change #161, changed-line coverage #162) run."""
    return (
        re.search(r"colocated-test .*--base", text) is not None
        and re.search(r"unit coverage .*--base", text) is not None
    )
