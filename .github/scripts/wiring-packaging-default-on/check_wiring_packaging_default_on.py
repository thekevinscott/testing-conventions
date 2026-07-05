#!/usr/bin/env python3
"""Assert packaging is default-on: it gates on detect's `packaging_dist` (#186; epic #302, #305).

Repo-only: this module exists *only in this repository*; it is not part of the published
`testing-conventions` tool. It backs the `wiring-packaging-default-on` job of the self-test
workflow (.github/workflows/testing-conventions-selftest.yml), which asserts that
.github/workflows/testing-conventions.yml gates the packaging job on the detect action's
`packaging_dist` output — so packaging runs by default.

It lives as a standalone, colocated-tested script — rather than an inline `run: |` block — so the
wiring check carries real unit + e2e tests, and so its match pattern escapes the GitHub-Actions
`${{ }}` templating trap (a `run:` body is templated before the shell ever sees it).
"""
from __future__ import annotations

import sys
from pathlib import Path

DEFAULT_WORKFLOW = ".github/workflows/testing-conventions.yml"

# The detect output the packaging job must reference to be default-on.
_GATE_OUTPUT = "packaging_dist"


def find_missing_wiring(text: str) -> str | None:
    """Error message when packaging does not gate on `packaging_dist`, else None."""
    if _GATE_OUTPUT in text:
        return None
    return (
        "the packaging job doesn't gate on detect's `packaging_dist` — "
        "packaging isn't default-on (#186)"
    )


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else Path(DEFAULT_WORKFLOW)
    problem = find_missing_wiring(path.read_text())
    if problem is not None:
        print(f"::error::{problem}")
        return 1
    print("packaging is default-on (gates on packaging_dist)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
