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
    crates = [package for package in packages if package.get("kind") == "crates"]
    if len(crates) != 1:
        return f"{len(crates)} packages have kind `crates`, so the CLI crate is ambiguous"
    crate = crates[0]
    glob = f"{crate['path']}/**"
    published = [package for package in packages if package.get("kind") in PUBLISHED_KINDS]
    if not published:
        return f"nothing is published to {' or '.join(PUBLISHED_KINDS)}"
    for package in published:
        name = package["name"]
        if glob not in package.get("globs", []):
            return f"`{name}` omits `{glob}` from its globs, so a CLI change leaves its version behind"
        if crate["name"] not in package.get("depends_on", []):
            return f"`{name}` does not depend on `{crate['name']}`, so its build can take a stale binary"
    return None
