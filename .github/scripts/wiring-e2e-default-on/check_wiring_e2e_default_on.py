#!/usr/bin/env python3
"""Assert e2e verify is default-on: it gates on detect's `e2e_attestation` (#186; epic #302, #305).

Repo-only: this module exists *only in this repository*; it is not part of the published
`testing-conventions` tool. It backs the `wiring-e2e-default-on` job of the self-test workflow
(.github/workflows/testing-conventions-selftest.yml), which asserts that
.github/workflows/testing-conventions.yml gates the e2e-verify job on the detect action's
`e2e_attestation` output — so e2e verify runs by default.

It lives as a standalone, colocated-tested script — rather than an inline `run: |` block — so the
wiring check carries real unit + e2e tests, and so its match pattern escapes the GitHub-Actions
`${{ }}` templating trap (a `run:` body is templated before the shell ever sees it).
"""
from __future__ import annotations

import sys
from pathlib import Path

DEFAULT_WORKFLOW = ".github/workflows/testing-conventions.yml"

# The detect output the e2e-verify job must reference to be default-on.
_GATE_OUTPUT = "e2e_attestation"


def find_missing_wiring(text: str) -> str | None:
    """Error message when e2e verify does not gate on `e2e_attestation`, else None."""
    if _GATE_OUTPUT in text:
        return None
    return (
        "the e2e-verify job doesn't gate on detect's `e2e_attestation` — "
        "e2e verify isn't default-on (#186)"
    )


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else Path(DEFAULT_WORKFLOW)
    problem = find_missing_wiring(path.read_text())
    if problem is not None:
        print(f"::error::{problem}")
        return 1
    print("e2e verify is default-on (gates on e2e_attestation)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
