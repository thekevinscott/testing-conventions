#!/usr/bin/env python3
"""Assert the reusable workflow's packaging job is wired to the derived package root (#280).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `packaging-package-root-wired` job of
`.github/workflows/testing-conventions-selftest.yml` (epic #302, sub-issue #307).

The packaging job must reference `needs.detect.outputs.package_root`, or a per-package `dist/`
the detect action found is never handed to the check step — it keeps inspecting the checkout
root's `dist/`. The reference confirms the derived package root reaches the packaging check.

Standalone script rather than inline `run:` bash so the assertion carries real colocated and
e2e tests, and so the grep pattern stays in source the GitHub Actions `${{ }}` templating pass
never rewrites — an inline `run:` block is templated before the shell runs, silently stripping
a literal `${{ ... }}` embedded in a grep pattern.
"""
from __future__ import annotations

import sys
from pathlib import Path

WIRING = "needs.detect.outputs.package_root"
DEFAULT_WORKFLOW = ".github/workflows/testing-conventions.yml"
ERROR = (
    "the reusable workflow's packaging job does not reference "
    "needs.detect.outputs.package_root — a per-package dist/ is never discovered, only a "
    "checkout-root one (#280)"
)
SUCCESS = "reusable workflow's packaging job reads the derived package_root"


def find_missing_wiring(text: str) -> str | None:
    """Return the error message when the workflow lacks the wiring reference, else None."""
    if WIRING in text:
        return None
    return ERROR


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else Path(DEFAULT_WORKFLOW)
    text = path.read_text()
    missing = find_missing_wiring(text)
    if missing is not None:
        print(f"::error::{missing}")
        return 1
    print(SUCCESS)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
