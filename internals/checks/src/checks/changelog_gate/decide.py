"""The changelog-gate path vocabulary and its first two decisions — repo-only.

A PR that changes public source under `packages/<pkg>/` must add one fragment to each of that
package's fragment directories: `packages/<pkg>/changelog.d/` and `packages/<pkg>/migrations.d/`,
named `YYYY-MM-DD-<slug>.md`. The fragment layout exists so concurrent PRs stop conflicting: a
shared file that every PR appends to at the same anchor collides by construction; a fragment is a
new file, so two PRs against one package add two different paths.
"""
from __future__ import annotations

import re
from collections.abc import Iterable

ROOT = "packages"

# The two fragment kinds, in the order the gate reports them missing.
KINDS = ("changelog", "migrations")

# Scanned over raw commit bodies rather than git's trailer parser, so the bypass works from any
# line of any commit — a blank line before it does not split it off as git trailers require.
_SKIP_LINE = re.compile(r"(?im)^skip-changelog:")


def has_skip_line(commit_messages: str) -> bool:
    """True when any commit body has a line opening with `skip-changelog:`."""
    return _SKIP_LINE.search(commit_messages) is not None


def changed_packages(changed: Iterable[str]) -> list[str]:
    """The package directories (`packages/<name>`) the paths touch, unique and sorted."""
    return sorted(
        {
            f"{parts[0]}/{parts[1]}"
            for path in changed
            if len(parts := path.split("/")) > 2 and parts[0] == ROOT
        }
    )
