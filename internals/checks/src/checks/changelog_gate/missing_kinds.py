"""The fragment kinds a changed package still owes."""
from __future__ import annotations

from collections.abc import Iterable

from checks.changelog_gate.decide import KINDS
from checks.changelog_gate.fragment import FRAGMENT_NAME, fragment


def missing_kinds(added: Iterable[str], pkg: str) -> list[str]:
    """The fragment kinds `pkg` still owes, in `KINDS` order.

    Only *added* paths count: editing an existing fragment is not a new entry.
    """
    present = {
        frag[1]
        for path in added
        if (frag := fragment(path)) is not None
        and frag[0] == pkg
        and FRAGMENT_NAME.fullmatch(frag[2]) is not None
    }
    return [kind for kind in KINDS if kind not in present]
