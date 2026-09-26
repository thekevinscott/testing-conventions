"""The repo-relative paths one instructions document imports — pure."""
from __future__ import annotations

import posixpath
import re

_FENCE = re.compile(r"^\s{0,3}(?:```|~~~)")
_CODE_SPAN = re.compile(r"`[^`]*`")
# An import opens a word: a mid-word `@` is an email address or a scoped package name.
_IMPORT = re.compile(r"(?:^|(?<=\s))@(\S+)")
# A `~` or absolute target reads a file no checkout holds, so counting it would make the verdict
# depend on the machine the gate ran on.
_OUTSIDE_ROOT = ("~", "/")


def imported_paths(containing: str, text: str) -> list[str]:
    """Every `@path` `containing` imports, in load order, as a path under the repo root.

    Resolved against the importing file's own directory. A fenced block, a code span, a mid-word
    `@`, and a target that escapes the checkout all yield nothing.
    """
    base = posixpath.dirname(containing)
    paths: list[str] = []
    fenced = False
    for line in text.splitlines():
        if _FENCE.match(line):
            fenced = not fenced
            continue
        if fenced:
            continue
        for match in _IMPORT.finditer(_CODE_SPAN.sub("", line)):
            target = match.group(1)
            if target.startswith(_OUTSIDE_ROOT):
                continue
            resolved = posixpath.normpath(posixpath.join(base, target))
            if ".." not in resolved.split("/"):
                paths.append(resolved)
    return paths
