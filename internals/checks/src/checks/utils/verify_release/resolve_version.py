"""Resolve the just-published npm version at a release commit."""
from __future__ import annotations

import subprocess

from checks.utils.verify_release.published_version import NPM_TAG_PREFIX, published_version
from checks.utils.verify_release.run_text import run_text


def resolve_version(sha: str, run=subprocess.run) -> str:
    """The just-published npm version pinned from the `testing-conventions-npm-v*` tags at `sha`."""
    out = run_text(run, ["git", "tag", "--merged", sha, "--list", f"{NPM_TAG_PREFIX}*"])
    return published_version([line.strip() for line in out.splitlines() if line.strip()])
