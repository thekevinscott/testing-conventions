"""The paths one instructions document imports — pure."""
from __future__ import annotations

import re
from pathlib import Path

_FENCE = re.compile(r"^\s{0,3}(?:```|~~~)")
_CODE_SPAN = re.compile(r"`[^`]*`")
# An import opens a word: a mid-word `@` is an email address or a scoped package name.
_IMPORT = re.compile(r"(?:^|(?<=\s))@(\S+)")


def imported_paths(containing: Path, text: str) -> list[Path]:
    """Every `@path` `containing` imports, in load order, resolved against its own directory.

    A fenced block, a code span, and a mid-word `@` all yield nothing.
    """
    paths: list[Path] = []
    fenced = False
    for line in text.splitlines():
        if _FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        for match in _IMPORT.finditer(_CODE_SPAN.sub("", line)):
            # Joining an absolute target onto the base discards the base, so one expression covers both.
            paths.append((containing.parent / Path(match.group(1)).expanduser()).resolve())
    return paths
