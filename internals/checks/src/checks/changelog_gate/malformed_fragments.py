"""Fragment paths whose filenames break the naming convention."""
from __future__ import annotations

from collections.abc import Iterable

from checks.changelog_gate.fragment import FRAGMENT_NAME, fragment


def malformed_fragments(changed: Iterable[str]) -> list[str]:
    """Touched fragment paths whose filenames break the naming convention.

    Each fragment directory carries a `README.md` describing the convention, which is not an entry.
    """
    return [
        path
        for path in changed
        if (frag := fragment(path)) is not None
        and frag[2] != "README.md"
        and FRAGMENT_NAME.fullmatch(frag[2]) is None
    ]
