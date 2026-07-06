"""Extract one job's own YAML region from a workflow file — repo-only (#278 #279, #321).

Shared by the coverage- and mutation-package-root-wired checks: both confine their
`needs.detect.outputs.package_root` search to a single job so a reference in a neighbouring job
can't satisfy the check. The region opens on the `  <start>:` job header (included) and closes at
the next `  <end>:` job header (excluded).
"""
from __future__ import annotations


def extract_job_block(text: str, start_header: str, end_header: str) -> str:
    """Return the YAML lines of the `start_header` job, excluding the `end_header` line.

    The region opens on the first line beginning `  <start>:` (included) and closes at the first
    line beginning `  <end>:` (excluded), so a reference belonging to the next job stays out.
    """
    start = f"  {start_header}:"
    end = f"  {end_header}:"
    block: list[str] = []
    inside = False
    for line in text.splitlines():
        if line.startswith(start):
            inside = True
        if line.startswith(end):
            inside = False
        if inside:
            block.append(line)
    return "\n".join(block)
