#!/usr/bin/env python3
"""Advance the moving major tag `@v0` to a released commit, forward-only.
Arguments: the released commit to advance to (required), then the tag to move (default 'v0').
"""
from __future__ import annotations

import sys

from advance import advance


def main(argv: list[str]) -> int:
    # Padded so an omitted argument binds blank; a length check here leaves an equivalent mutant.
    sha, tag, *_ = (*argv, "", "")
    sha = sha.strip()
    if not sha:
        print("::error::a commit SHA is required (the released commit to advance the tag to)")
        return 1
    tag = tag.strip() or "v0"
    action = advance(tag, sha)
    print({
        "bootstrap": f"{tag} did not exist yet; bootstrapped it at {sha}",
        "advance": f"advanced {tag} -> {sha}",
        "noop": f"{tag} is already at or ahead of {sha}; nothing to do",
    }[action])
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
