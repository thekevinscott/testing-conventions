#!/usr/bin/env python3
"""Assert the reusable workflow offers the opt-in `e2e verify` job — `e2e-verify-wired` (#173).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `e2e-verify-wired` job of
`.github/workflows/testing-conventions-selftest.yml`, which guards the red→green signal for
#173 — the workflow must offer the e2e freshness gate (#68) behind an opt-in `run_e2e` input, or
consumers cannot enforce it on the `@v0` path. Both the input (`run_e2e`) and the command
(`e2e verify`) must be present.

It lives here, rather than as an inline `run: |` block, so it carries real colocated + e2e tests.
Inline `run:` bash is untested prose and is exposed to the GitHub-Actions `${{ }}` templating
trap (a `run:` body is templated before the shell sees it, so a literal `${{ ... }}` inside a
grep pattern is silently evaluated away). A standalone Python script sidesteps both.

Stdlib only.
"""
from __future__ import annotations

import re
import sys
from pathlib import Path
from typing import Optional

DEFAULT_PATH = ".github/workflows/testing-conventions.yml"
INPUT_MARKER = "run_e2e"
COMMAND = re.compile(r"e2e verify")


def find_missing_wiring(text: str) -> Optional[str]:
    """Return an error message unless both the `run_e2e` input and `e2e verify` command appear."""
    if INPUT_MARKER not in text or COMMAND.search(text) is None:
        return (
            "the reusable workflow does not offer an opt-in `e2e verify` job (input `run_e2e` "
            "+ `e2e verify`) — the #68 freshness gate isn't on the @v0 path (#173)"
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
    print("reusable workflow offers opt-in e2e verify")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
