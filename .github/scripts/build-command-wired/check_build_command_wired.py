#!/usr/bin/env python3
"""Assert the reusable workflow wires a `build_command` step — repo-only (#302, #306).

Repo-only: this module exists *only in this repository*; it is not part of the published
`testing-conventions` tool. It backs the `build-command-wired` job in
`.github/workflows/testing-conventions-selftest.yml`, the red->green wiring signal for #243:
the reusable workflow (`.github/workflows/testing-conventions.yml`) runs a derived
`build_command` step — read from detect's `build_command` output (the package's own
`[python].build_command`, #289) — before the suite-executing jobs, so a native-binding
consumer's compiled module is built before the suite imports it.

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
REQUIRED = "needs.detect.outputs.build_command"
ERROR = (
    "the reusable workflow has no derived `build_command` step — a native-binding consumer's "
    "compiled module is never built before the suite runs (#243/#289)"
)
SUCCESS = "reusable workflow wires a build_command step"


def find_missing_wiring(text: str) -> Optional[str]:
    """Return the error message when `text` lacks the `build_command` wiring, else None."""
    if REQUIRED in text:
        return None
    return ERROR


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if argv[1:] else Path(DEFAULT_WORKFLOW)
    problem = find_missing_wiring(path.read_text())
    if problem is not None:
        print(f"::error::{problem}")
        return 1
    print(SUCCESS)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
