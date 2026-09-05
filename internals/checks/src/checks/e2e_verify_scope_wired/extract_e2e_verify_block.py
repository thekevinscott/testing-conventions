"""Extraction of the `e2e-verify:` job's own YAML block from the reusable workflow."""
from __future__ import annotations

import re

_JOB_START = re.compile(r"^  e2e-verify:", re.MULTILINE)
_NEXT_JOB = re.compile(r"^  packaging:", re.MULTILINE)


def extract_e2e_verify_block(workflow_text: str) -> str:
    """The `e2e-verify:` job's own YAML, up to (not including) the next top-level job."""
    start = _JOB_START.search(workflow_text)
    if not start:
        return ""
    end = _NEXT_JOB.search(workflow_text, start.end())
    return workflow_text[start.start() : end.start() if end else len(workflow_text)]
