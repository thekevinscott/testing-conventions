"""The `@path` imports one instructions document declares — pure."""
from __future__ import annotations

import re

_FENCE = re.compile(r"^\s{0,3}(?:```|~~~)")
_CODE_SPAN = re.compile(r"`[^`]*`")
# An import opens a word: a mid-word `@` is an email address or a scoped package name.
_IMPORT = re.compile(r"(?:^|(?<=\s))@(\S+)")


def import_targets(text: str) -> list[str]:
    """Every `@path` the document imports, in order, skipping fenced blocks and code spans."""
    targets: list[str] = []
    fenced = False
    for line in text.splitlines():
        if _FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        targets.extend(match.group(1) for match in _IMPORT.finditer(_CODE_SPAN.sub("", line)))
    return targets
