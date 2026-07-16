"""The suite-toolchain-pinned-wired decision — repo-only (#438).

The reusable workflow provisions a consumer's Python suite toolchain with `uv pip install coverage
pytest testing-conventions`, and that line runs in the **consumer's** required check. Left floating,
a new `coverage`/`pytest` release changes what the consumer's gate runs with no commit to blame —
the #437 hermeticity gap, on the consumer surface. The #437 lock can't close it: `.github/uv/
engines.txt` lives in *this* repo, and the reusable workflow ships frozen at `@v0` into the
consumer's checkout, where that file does not exist. The pin that travels is one written into the
workflow YAML itself, so the decision holds one fact: every `uv pip install` line pins each suite
engine to an exact `==` version.

`testing-conventions` on that same line is deliberately unpinned — it is the package under test
(the published wheel, or the HEAD build in hermetic mode, #356), not a third-party engine — so the
decision is scoped to `coverage` and `pytest`.
"""
from __future__ import annotations

import re

# The consumer suite engines the reusable workflow installs — third-party, so pinned. Not
# `cosmic-ray`/`maturin`: those are the repo-internal toolchain (#437), layered by other jobs.
SUITE_ENGINES = ("coverage", "pytest")

_INSTALL = "uv pip install"
_ALT = "|".join(SUITE_ENGINES)
# A suite engine NOT immediately followed by `==` is unpinned (bare, or a floating range like `>=`).
_UNPINNED = re.compile(rf"\b(?:{_ALT})\b(?!==)")
# A suite engine pinned to an exact version.
_PINNED = re.compile(rf"\b({_ALT})==")


def _install_lines(text: str) -> list[str]:
    """Every `uv pip install` invocation in `text`, comment lines excluded."""
    return [
        line
        for line in text.splitlines()
        if _INSTALL in line and not line.lstrip().startswith("#")
    ]


def unpinned_engines(text: str) -> list[str]:
    """Every suite engine left unpinned in a `uv pip install` line."""
    return [match for line in _install_lines(text) for match in _UNPINNED.findall(line)]


def decide(text: str) -> bool:
    """True when every suite engine is pinned to an exact `==` version in every install line, and
    none is left floating."""
    pinned = {match for line in _install_lines(text) for match in _PINNED.findall(line)}
    return not unpinned_engines(text) and all(engine in pinned for engine in SUITE_ENGINES)
