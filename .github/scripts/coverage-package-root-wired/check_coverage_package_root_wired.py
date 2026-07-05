#!/usr/bin/env python3
"""Assert the reusable workflow's coverage jobs are wired to the derived package root (#278).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `coverage-package-root-wired` job of
`.github/workflows/testing-conventions-selftest.yml` (epic #302, sub-issue #307).

Both coverage jobs — `unit-coverage` and `coverage-changed` — must reference
`needs.detect.outputs.package_root` (the #277 primitive) so their TS install, Python-env
provisioning, and `build_command` step run at the derived package root, not the checkout root.
The check isolates each job's own YAML region and confirms the reference lives inside it.

Standalone script rather than inline `run:` bash so the assertion carries real colocated and
e2e tests, and so the grep pattern stays in source the GitHub Actions `${{ }}` templating pass
never rewrites — an inline `run:` block is templated before the shell runs, silently stripping
a literal `${{ ... }}` embedded in a grep pattern.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Optional

WIRING = "needs.detect.outputs.package_root"
DEFAULT_WORKFLOW = ".github/workflows/testing-conventions.yml"

# (job header, next job header, error message, success message). The next header bounds the
# job's YAML region, mirroring the selftest's `awk '/^  <job>:/,/^  <next>:/'` range.
_JOBS = (
    (
        "unit-coverage",
        "coverage-changed",
        "the unit-coverage job doesn't reference needs.detect.outputs.package_root — "
        "TS install / Python env / build_command still run at the checkout root, not the "
        "derived package root (#278)",
        "unit-coverage installs/builds at needs.detect.outputs.package_root",
    ),
    (
        "coverage-changed",
        "mutation",
        "the coverage-changed job doesn't reference needs.detect.outputs.package_root — "
        "TS install / Python env / build_command still run at the checkout root, not the "
        "derived package root (#278)",
        "coverage-changed installs/builds at needs.detect.outputs.package_root",
    ),
)


def extract_job_block(text: str, start_header: str, end_header: str) -> str:
    """Return the YAML lines of the `start_header` job, up to and including the `end_header`.

    Mirrors the inline `awk '/^  <start>:/,/^  <end>:/'` range: the block opens on the first
    line beginning `  <start>:` and closes on the first following line beginning `  <end>:`
    (both boundary lines included). An absent end header yields the block through end of file.
    """
    start = f"  {start_header}:"
    end = f"  {end_header}:"
    block: list[str] = []
    inside = False
    for line in text.splitlines():
        if not inside and line.startswith(start):
            inside = True
            block.append(line)
            continue
        if inside:
            block.append(line)
            if line.startswith(end):
                break
    return "\n".join(block)


def find_missing_wiring(text: str) -> Optional[str]:
    """Return the error message for the first coverage job missing the wiring, else None."""
    for start, end, error, _success in _JOBS:
        block = extract_job_block(text, start, end)
        if WIRING not in block:
            return error
    return None


def main(argv: list[str]) -> int:
    path = Path(argv[1]) if len(argv) > 1 else Path(DEFAULT_WORKFLOW)
    text = path.read_text()
    missing = find_missing_wiring(text)
    if missing is not None:
        print(f"::error::{missing}")
        return 1
    for _start, _end, _error, success in _JOBS:
        print(success)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
