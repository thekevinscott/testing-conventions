"""Find the jobs in a workflow that invoke the published CLI."""
from __future__ import annotations

from checks.utils.job_block import iter_job_blocks

# The package spec, not the `npx` prefix: the launcher around it changes, the spec is the invariant.
CLI_INVOCATION = '"testing-conventions${VERSION:+@$VERSION}"'


def cli_jobs(text: str) -> list[tuple[str, str]]:
    """The `(name, block)` jobs in workflow `text` that invoke the published CLI."""
    return [(name, block) for name, block in iter_job_blocks(text) if CLI_INVOCATION in block]
