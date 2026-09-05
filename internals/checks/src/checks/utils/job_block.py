"""Discover every job in a workflow's `jobs:` mapping and bound each to its own YAML region,
for a check that doesn't know the job names up front."""
from __future__ import annotations

import re

_JOB_HEADER = re.compile(r"^  ([A-Za-z][\w-]*):[ \t]*$", re.M)


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
