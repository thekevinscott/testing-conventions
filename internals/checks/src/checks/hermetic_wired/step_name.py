"""A step block's display name, for error messages that point at one step."""
from __future__ import annotations

import re

NAME_LINE = re.compile(r"^      - name: (.*)$", re.M)


def step_name(block: str) -> str:
    """A step's `name:`, or its opening line when it declares none."""
    named = NAME_LINE.search(block)
    return named.group(1) if named else block.split("\n")[0].strip().removeprefix("- ")
