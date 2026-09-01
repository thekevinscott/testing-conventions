"""Bound each `steps:` list item to its own lines, so the `${CLI_COMMAND:-` fallback is checked
per step rather than file-wide.

`CLI_COMMAND` is a step-local `env:` value: a step running the fallback without its own
`CLI_COMMAND: ${{ needs.detect.outputs.cli_command }}` line expands to the published binary
while every other step still carries the fallback text, so a file-wide substring check reads nine
wired steps out of ten as fully wired.
"""
from __future__ import annotations

import re

ENV_VALUE = "CLI_COMMAND: ${{ needs.detect.outputs.cli_command }}"
FALLBACK = "${CLI_COMMAND:-"
STEP_BODY = "      "
STEP_START = re.compile(r"(?m)^(?=      - )")
NAME_LINE = re.compile(r"^      - name: (.*)$", re.M)


def step_blocks(text: str) -> list[str]:
    """Every `steps:` list item in `text`, bounded to the lines that belong to it.

    A chunk ends at the next list item (the split point) or at the first line indented shallower
    than a step body — the job-level key that closes the `steps:` list.
    """
    blocks = []
    for chunk in STEP_START.split(text)[1:]:
        lines = chunk.split("\n")
        ends = (i for i, line in enumerate(lines) if line.strip() and not line.startswith(STEP_BODY))
        blocks.append("\n".join(lines[: next(ends, len(lines))]))
    return blocks


def step_name(block: str) -> str:
    """A step's `name:`, or its opening line when it declares none."""
    named = NAME_LINE.search(block)
    return named.group(1) if named else block.split("\n")[0].strip().removeprefix("- ")


def unwired_steps(text: str) -> list[str]:
    """Names of the steps running the fallback without their own `CLI_COMMAND` env line."""
    running = [block for block in step_blocks(text) if FALLBACK in block]
    return [step_name(block) for block in running if ENV_VALUE not in block]
