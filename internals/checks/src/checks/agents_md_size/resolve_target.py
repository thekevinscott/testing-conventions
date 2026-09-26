"""The repo-relative path an import target names, resolved against its containing file — pure."""
from __future__ import annotations

import posixpath
from typing import Optional

_OUTSIDE_ROOT = ("~", "/")


def resolve_target(containing: str, target: str) -> Optional[str]:
    """`target` as a path under the repo root, or None when it names a file outside the checkout.

    A `~`, absolute, or upward target reads a file that varies by machine, so counting it would
    make the gate's verdict depend on where it runs.
    """
    if target.startswith(_OUTSIDE_ROOT):
        return None
    resolved = posixpath.normpath(posixpath.join(posixpath.dirname(containing), target))
    if ".." in resolved.split("/"):
        return None
    return resolved
