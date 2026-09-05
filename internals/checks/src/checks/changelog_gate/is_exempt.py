"""The paths a package may change without owing a fragment."""
from __future__ import annotations

import re

from checks.changelog_gate.decide import KINDS


def is_exempt(path: str, pkg: str) -> bool:
    """True when `path` is not source: a frozen stub, a fragment, a receipt, or a test.

    `CHANGELOG.md` / `MIGRATIONS.md` are the frozen pre-fragment archive and the `changelog.d/` /
    `migrations.d/` folders are the entries themselves, so neither can be the change that demands
    an entry. `e2e-attestations/` holds CI freshness receipts written by automation. The test
    shapes are this repo's three: a `tests/` suite tier, a colocated Python `*_test.py`, and a
    colocated TypeScript `*.test.*` / `*.spec.*`.
    """
    p = re.escape(pkg)
    return bool(
        re.fullmatch(rf"{p}/(CHANGELOG|MIGRATIONS)\.md", path)
        or re.match(rf"{p}/({'|'.join(KINDS)})\.d/", path)
        or re.match(rf"{p}/e2e-attestations/", path)
        or re.match(rf"{p}/(tests?|__tests__)/", path)
        or re.fullmatch(rf"{p}/.*_test\.py", path)
        or re.fullmatch(rf"{p}/.*\.(test|spec)\.(ts|tsx|js|mjs|cjs|py|rs)", path)
    )
