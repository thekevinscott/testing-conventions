#!/usr/bin/env python3
"""Assert the reusable workflow offers a `gates` allowlist input — repo-only (#302, #306).

Repo-only: this module exists *only in this repository*; it is not part of the published
`testing-conventions` tool. It backs the `gates-wired` job in
`.github/workflows/testing-conventions-selftest.yml`, the red->green wiring signal for #262:
the reusable workflow (`.github/workflows/testing-conventions.yml`) offers a `gates` input that
restricts which gates run, so a consumer driving the suite-executing gates in its own
build-capable jobs can adopt the workflow for just the static gates.

It is a standalone, colocated-tested script rather than an inline `run: |` block because inline
workflow bash is untested prose and is exposed to the GitHub Actions `${{ }}` templating trap:
the `run:` text is templated before the shell sees it, so a literal `${{ ... }}` in a grep
pattern is silently evaluated. Extracting the assertion into Python gives it real unit + e2e
coverage (#302, sub-issue #306).
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Optional

DEFAULT_WORKFLOW = ".github/workflows/testing-conventions.yml"
REQUIRED = "inputs.gates"
ERROR = (
    "the reusable workflow has no `gates` input — gate selection is all-or-nothing, so a "
    "consumer that runs the suite-executing gates in its own build-capable jobs cannot adopt "
    "the workflow for the static gates (#262)"
)
SUCCESS = "reusable workflow scopes its gates to the `gates` allowlist"


def find_missing_wiring(text: str) -> Optional[str]:
    """Return the error message when `text` lacks the `gates` allowlist wiring, else None."""
    if REQUIRED in text:
        return None
    return ERROR


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else Path(DEFAULT_WORKFLOW)
    problem = find_missing_wiring(path.read_text())
    if problem is not None:
        print(f"::error::{problem}")
        return 1
    print(SUCCESS)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
