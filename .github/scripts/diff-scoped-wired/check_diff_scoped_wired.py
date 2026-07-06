#!/usr/bin/env python3
"""Assert the reusable workflow runs the diff-scoped `--base` checks — `diff-scoped-wired` (#172).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `diff-scoped-wired` job of
`.github/workflows/testing-conventions-selftest.yml`, which guards the red→green signal for
#172 — the workflow must invoke both the commit-scoped co-change check (`unit colocated-test
--base`, #161) and the changed-line coverage check (`unit coverage --base`, #162), or neither
diff-scoped rule runs in CI.

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
CO_CHANGE = re.compile(r"colocated-test .*--base")
CHANGED_LINE_COVERAGE = re.compile(r"unit coverage .*--base")


def find_missing_wiring(text: str) -> Optional[str]:
    """Return an error message if either diff-scoped `--base` check is not invoked.

    Both the co-change (`colocated-test --base`) and changed-line coverage (`unit coverage
    --base`) invocations must be present; a missing either one means that diff-scoped rule
    never runs in CI.
    """
    if CO_CHANGE.search(text) is None or CHANGED_LINE_COVERAGE.search(text) is None:
        return (
            "the reusable workflow does not run the diff-scoped `--base` checks (co-change "
            "#161 / changed-line coverage #162) — they never run in CI (#172)"
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
    print("reusable workflow runs the diff-scoped --base checks")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
