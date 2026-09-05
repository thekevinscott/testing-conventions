"""Whether a package's diff holds real source changes."""
from __future__ import annotations

from collections.abc import Iterable

from checks.changelog_gate.is_exempt import is_exempt


def code_touched(changed: Iterable[str], pkg: str) -> bool:
    """True when `pkg` has source changes — anything under it that is not exempt."""
    return any(
        path.startswith(f"{pkg}/") and not is_exempt(path, pkg) for path in changed
    )
