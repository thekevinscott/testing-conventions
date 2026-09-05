"""Extract one named job's own YAML region from a workflow file."""
from __future__ import annotations


def extract_job_block(text: str, start_header: str, end_header: str) -> str:
    """The YAML lines from the first `  <start>:` line (included) to the first `  <end>:` line
    (excluded), so a reference belonging to the next job stays out."""
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
