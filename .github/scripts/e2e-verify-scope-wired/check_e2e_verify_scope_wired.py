#!/usr/bin/env python3
"""Assert the e2e-verify job passes --scope naming inputs.path (#294).

Repo-only: this module exists *only in this repository*. It backs the
`e2e-verify-scoped-to-path` job in `testing-conventions-selftest.yml` — a red->green wiring
signal that the reusable workflow's `e2e verify` invocation scopes the freshness walk to the
caller's own `path` input, not the (possibly broader) derived `package_root`. A static check
against the workflow file, so it tracks the workflow regardless of what the published binary
ships; runtime behavior is covered by the Rust e2e tests (`e2e_verify.rs`, `e2e_verify_e2e.rs`).

Written as a script with its own unit test rather than inline in the workflow YAML: an inline
`run: |` block is untested prose, and GitHub Actions templates a workflow's `run:` text for
`${{ }}` expressions before the shell ever sees it — embedding a literal `${{ inputs.path }}`
inside a grep pattern gets silently evaluated (and stripped) by that templating, which is
exactly the bug this script's own test suite would have caught before it reached CI.

Usage: check_e2e_verify_scope_wired.py [path-to-testing-conventions.yml]
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

DEFAULT_WORKFLOW = ".github/workflows/testing-conventions.yml"

_JOB_START = re.compile(r"^  e2e-verify:", re.MULTILINE)
_NEXT_JOB = re.compile(r"^  packaging:", re.MULTILINE)
_HAS_SCOPE_FLAG = re.compile(r"--scope")
_HAS_SCAN_PATH_FROM_INPUTS_PATH = re.compile(r"SCAN_PATH:\s*.*inputs\.path")


def extract_e2e_verify_block(workflow_text: str) -> str:
    """The `e2e-verify:` job's own YAML, up to (not including) the next top-level job."""
    start = _JOB_START.search(workflow_text)
    if not start:
        return ""
    end = _NEXT_JOB.search(workflow_text, start.end())
    return workflow_text[start.start(): end.start() if end else len(workflow_text)]


def find_missing_wiring(workflow_text: str) -> str | None:
    """None if the e2e-verify job passes --scope naming inputs.path; else an error message."""
    block = extract_e2e_verify_block(workflow_text)
    if not _HAS_SCOPE_FLAG.search(block) or not _HAS_SCAN_PATH_FROM_INPUTS_PATH.search(block):
        return (
            "the e2e-verify job doesn't pass --scope naming inputs.path — the freshness walk "
            "is scoped to the derived package_root instead, which can be broader than what the "
            "caller's own path input names (#294)"
        )
    return None


def main(argv: list[str]) -> int:
    workflow_path = Path(argv[1] if len(argv) > 1 else DEFAULT_WORKFLOW)
    error = find_missing_wiring(workflow_path.read_text())
    if error:
        print(f"::error::{error}")
        return 1
    print("e2e-verify scopes the freshness walk to inputs.path")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
