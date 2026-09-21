"""The release-lockstep decision — repo-only.

A consumer job installs the npm CLI and the PyPI plugin by name, so the pair it gets is coherent
only while both published packages rebuild from the CLI crate. See docs/internals/repo.md,
"Registry skew: the version numbers differ, the rules don't".
"""
from __future__ import annotations

from typing import Optional

PUBLISHED_KINDS = ("npm", "pypi")


def lockstep_error(packages) -> Optional[str]:
    """The first way `packages` — putitoutthere's `[[package]]` list — lets the published halves
    drift apart, or `None` when every one of them rebuilds from the CLI crate."""
    raise NotImplementedError
