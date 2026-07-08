"""Extract one job's own YAML region from a workflow file — repo-only (#278 #279, #321, #356).

Shared by the coverage- and mutation-package-root-wired checks: both confine their
`needs.detect.outputs.package_root` search to a single job so a reference in a neighbouring job
can't satisfy the check. The region opens on the `  <start>:` job header (included) and closes at
the next `  <end>:` job header (excluded).

`iter_job_blocks` generalizes this for a check that doesn't know every job name up front (#356's
hermetic-wired: a caller workflow's `uses:`-calling jobs are named freely, e.g. "clean",
"packaging-clean") — it discovers every job header itself and yields each job already bounded to
its own block, so a per-job assertion never has to know its neighbours' names in advance.
"""
from __future__ import annotations

import re

_JOB_HEADER = re.compile(r"^  ([A-Za-z][\w-]*):[ \t]*$", re.M)


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


def iter_job_blocks(text: str) -> list[tuple[str, str]]:
    """Split a workflow's `jobs:` mapping into `(name, block)` pairs, in file order.

    A job header is a line indented exactly two spaces, ending in a bare `:` — unlike
    `  contents: read` or `  group: ...`, which carry a value on the same line — so a scalar
    top-level key elsewhere in the file (permissions, concurrency) is never mistaken for a job.
    Each block runs to the next header, or to the end of the file for the last job.
    """
    headers = list(_JOB_HEADER.finditer(text))
    return [
        (m.group(1), text[m.start() : headers[i + 1].start() if i + 1 < len(headers) else len(text)])
        for i, m in enumerate(headers)
    ]
