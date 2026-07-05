#!/usr/bin/env python3
"""Assert the reusable workflow detects via the tested `detect` action (#185; epic #302, #305).

Repo-only: this module exists *only in this repository*; it is not part of the published
`testing-conventions` tool. It backs the `wiring-detect-action` job of the self-test workflow
(.github/workflows/testing-conventions-selftest.yml), which asserts that
.github/workflows/testing-conventions.yml wires detection to the `detect` action.

It lives as a standalone, colocated-tested script — rather than an inline `run: |` block — so the
wiring check carries real unit + e2e tests, and so its grep pattern escapes the GitHub-Actions
`${{ }}` templating trap (a `run:` body is templated before the shell ever sees it, which would
silently evaluate a literal `${{ ... }}` embedded in a pattern).

The workflow file to inspect is passed as `argv[1]`.
"""
import re
import sys
from pathlib import Path
from typing import Optional

# `uses: <maybe-prefix>actions/detect@<ref>` — the reusable workflow pins the detect action,
# mirroring the original grep `uses:[[:space:]]*[^[:space:]]*actions/detect@`.
_DETECT_ACTION = re.compile(r"uses:\s*\S*actions/detect@")


def find_missing_wiring(text: str) -> Optional[str]:
    """Error message when the workflow does not use the detect action, else None."""
    if _DETECT_ACTION.search(text):
        return None
    return (
        "the reusable workflow doesn't use the detect action — detection still runs as "
        "inline bash, off the tested engine (#185)"
    )


def main(argv: list) -> int:
    problem = find_missing_wiring(Path(argv[1]).read_text())
    if problem is not None:
        print(f"::error::{problem}")
        return 1
    print("reusable workflow detects via the detect action")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
