"""Extraction of the `packaging:` job's own YAML block from the reusable workflow."""
from __future__ import annotations

import re

_JOB_START = re.compile(r"^  packaging:", re.MULTILINE)
_NEXT_JOB = re.compile(r"^  [a-z][a-z0-9-]*:\s*$", re.MULTILINE)


def extract_packaging_block(workflow_text: str) -> str:
    """The `packaging:` job's own YAML, up to (not including) the next top-level job (or the end
    of the file, since packaging is currently the last job)."""
    start = _JOB_START.search(workflow_text)
    if not start:
        return ""
    end = _NEXT_JOB.search(workflow_text, start.end())
    return workflow_text[start.start() : end.start() if end else len(workflow_text)]
