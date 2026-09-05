"""Chunking of a workflow's `steps:` text into per-step line lists for the pnpm-setup scan."""
from __future__ import annotations

PNPM_SETUP = "pnpm/action-setup"


def pnpm_steps(text: str) -> list[list[str]]:
    """Every step chunk in `text` that uses `pnpm/action-setup` — a line opening a list item
    (`- `) through the line before the next one. Chunking on the step boundary keeps a
    version-less step from borrowing the next step's; lines before the first step are dropped."""
    chunks: list[list[str]] = []
    for line in text.splitlines():
        if line.lstrip().startswith("- "):
            chunks.append([])
        if chunks:
            chunks[-1].append(line)
    return [chunk for chunk in chunks if any(PNPM_SETUP in line for line in chunk)]
