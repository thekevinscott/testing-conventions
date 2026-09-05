#!/usr/bin/env python3
"""Advance the moving major tag `@v0` to a released commit, forward-only.
Inputs come from the environment the workflow sets: SHA (required), TAG (default 'v0').
"""
from __future__ import annotations

import os

from advance import advance


def main() -> int:
    sha = os.environ.get("SHA", "").strip()
    if not sha:
        print("::error::SHA is required (the released commit to advance the tag to)")
        return 1
    tag = os.environ.get("TAG", "v0").strip() or "v0"
    action = advance(tag, sha)
    print({
        "bootstrap": f"{tag} did not exist yet; bootstrapped it at {sha}",
        "advance": f"advanced {tag} -> {sha}",
        "noop": f"{tag} is already at or ahead of {sha}; nothing to do",
    }[action])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
