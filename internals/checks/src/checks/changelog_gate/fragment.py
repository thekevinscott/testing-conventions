"""Recognise a fragment path and its filename convention."""
from __future__ import annotations

import re
from typing import Optional

from checks.changelog_gate.decide import KINDS, ROOT

# Captures (<package dir>, <kind>, <filename>). The trailing `[^/]+` forbids nesting, so a stray
# subdirectory under a fragment dir is not a fragment.
_FRAGMENT = re.compile(rf"({ROOT}/[^/]+)/({'|'.join(KINDS)})\.d/([^/]+)")

# An ISO date, a kebab-case slug, and `.md`; the date is the UTC merge date.
FRAGMENT_NAME = re.compile(r"\d{4}-\d{2}-\d{2}-[a-z0-9-]+\.md")


def fragment(path: str) -> Optional[tuple[str, str, str]]:
    """`(package dir, kind, filename)` when `path` is a fragment, else `None`."""
    match = _FRAGMENT.fullmatch(path)
    return (match.group(1), match.group(2), match.group(3)) if match is not None else None
