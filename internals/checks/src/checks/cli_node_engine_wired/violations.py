"""The node-floor violations among a workflow's CLI-invoking jobs."""
from __future__ import annotations

import re

from checks.cli_node_engine_wired.cli_jobs import cli_jobs

# Anchored on `- uses:` because a step opening that way carries no `if:`: a gated step, which
# leaves the CLI on the runner's ambient node, must not satisfy the check.
SETUP_NODE = re.compile(
    r"^      - uses: actions/setup-node@[^\n]*\n(?:        [^\n]*\n)*?          node-version: (\d+)",
    re.M,
)


def violations(text: str, floor: int) -> list[str]:
    """The CLI-invoking jobs in workflow `text` that do not unconditionally provision node `floor`."""
    problems = []
    for name, block in cli_jobs(text):
        pinned = [int(major) for major in SETUP_NODE.findall(block)]
        if not pinned:
            problems.append(f"`{name}` invokes the CLI with no unconditional `setup-node` step")
        elif max(pinned) < floor:
            problems.append(f"`{name}` pins node {max(pinned)}, below the floor of {floor}")
    return problems
