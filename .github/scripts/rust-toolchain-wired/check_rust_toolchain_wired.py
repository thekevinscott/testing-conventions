#!/usr/bin/env python3
"""Assert the reusable workflow provisions a Rust toolchain input — repo-only (#302, #306).

Repo-only: this module exists *only in this repository*; it is not part of the published
`testing-conventions` tool. It backs the `rust-toolchain-wired` job in
`.github/workflows/testing-conventions-selftest.yml`, the red->green wiring signal for #263:
the suite-executing jobs (unit coverage, changed-line coverage, mutation) offer a
`rust_toolchain` input that provisions cargo before `build_command` runs, so a native binding
whose build compiles a Rust core has cargo available in the python/typescript arms.

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
REQUIRED = "inputs.rust_toolchain"
ERROR = (
    "the reusable workflow has no `rust_toolchain` input — a native binding whose build_command "
    "compiles a Rust core has no cargo in the python/typescript suite-executing jobs, so the "
    "build fails before the suite runs (#263)"
)
SUCCESS = "reusable workflow provisions a Rust toolchain behind `rust_toolchain`"


def find_missing_wiring(text: str) -> Optional[str]:
    """Return the error message when `text` lacks the `rust_toolchain` wiring, else None."""
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
