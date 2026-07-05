#!/usr/bin/env python3
"""Assert the reusable workflow's mutation job is wired to the derived package root (#279).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `mutation-package-root-wired` job of
`.github/workflows/testing-conventions-selftest.yml` (epic #302, sub-issue #307).

The `mutation` job must reference `needs.detect.outputs.package_root` — in its install steps,
its `build_command` step, and its Rust-build cache path — or it installs and builds at the
checkout root, so a per-package-lockfile monorepo (or a non-pnpm TS package, or a uv-managed
Python package) fails the job. The check isolates the `mutation` job's own YAML region — a
reference in a neighbouring job must not satisfy it — and confirms the wiring lives inside it.

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
JOB_HEADER = "mutation"
NEXT_JOB_HEADER = "integration-lint"
ERROR = (
    "the mutation job does not reference needs.detect.outputs.package_root — it still "
    "installs/builds at the checkout root, so a per-package-lockfile monorepo (or a non-pnpm "
    "TS package, or a uv-managed Python package) fails the job (#279)"
)
SUCCESS = "mutation job installs and builds from the derived package root"


def extract_job_block(text: str, start_header: str, end_header: str) -> str:
    """Return the YAML lines of the `start_header` job, excluding the `end_header` line.

    Mirrors the inline `awk '/^  <start>:/{flag=1} /^  <end>:/{flag=0} flag'` extraction: the
    region opens on the first line beginning `  <start>:` (included) and closes at the first
    line beginning `  <end>:` (excluded, since the flag clears before that line prints).
    """
    start = f"  {start_header}:"
    end = f"  {end_header}:"
    block: list[str] = []
    inside = False
    for line in text.splitlines():
        if line.startswith(start):
            inside = True
        if line.startswith(end):
            inside = False
        if inside:
            block.append(line)
    return "\n".join(block)


def find_missing_wiring(text: str) -> Optional[str]:
    """Return the error message when the mutation job lacks the wiring reference, else None."""
    block = extract_job_block(text, JOB_HEADER, NEXT_JOB_HEADER)
    if WIRING in block:
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
