#!/usr/bin/env python3
"""Assert the reusable workflow invokes `unit lint` — the `isolation-wired` self-test (#125).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `isolation-wired` job of
`.github/workflows/testing-conventions-selftest.yml`, which guards the red→green signal for
#125 — the reusable workflow must actually invoke `unit lint`, or the shipped isolation rule is
not enforced on the `@v0` path.

It lives here, rather than as an inline `run: |` block, so it carries real colocated + e2e tests.
Inline `run:` bash is untested prose, and it is exposed to the GitHub-Actions `${{ }}` templating
trap: a `run:` body is templated for `${{ }}` before the shell sees it, so a literal `${{ ... }}`
inside a grep pattern is silently evaluated away. A standalone Python script sidesteps both.

Stdlib only.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Optional

DEFAULT_PATH = ".github/workflows/testing-conventions.yml"
MARKER = "unit lint --language"


def find_missing_wiring(text: str) -> Optional[str]:
    """Return an error message if the reusable workflow does not invoke `unit lint`."""
    if MARKER not in text:
        return (
            "the reusable workflow does not invoke `unit lint` — the shipped lint rule is "
            "not enforced on the @v0 path (#125)"
        )
    return None


def path_from_argv(argv: list[str], default: str) -> str:
    """The workflow file to inspect: `argv[1]` when given, else the repo-relative default."""
    return argv[1] if argv[1:] else default


def main(argv: list[str]) -> int:
    path = path_from_argv(argv, DEFAULT_PATH)
    text = Path(path).read_text()
    missing = find_missing_wiring(text)
    if missing is not None:
        print(f"::error::{missing}")
        return 1
    print("reusable workflow invokes `unit lint`")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
