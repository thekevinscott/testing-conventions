"""Read the mutated source out of a cosmic-ray work result's unified diff."""
from __future__ import annotations


def replacement_from_diff(diff):
    """The source line(s) a work result's unified ``diff`` adds, or ``None`` when it adds none."""
    added = []
    in_hunk = False
    for line in (diff or "").splitlines():
        if line.startswith("@@"):
            in_hunk = True
        elif in_hunk and line.startswith("+"):
            added.append(line[1:].strip())
    return "\n".join(added) or None
