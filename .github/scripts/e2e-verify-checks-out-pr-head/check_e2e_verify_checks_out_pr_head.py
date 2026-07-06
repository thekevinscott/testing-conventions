#!/usr/bin/env python3
"""Assert e2e-verify checks out the PR head commit — `e2e-verify-checks-out-pr-head`.

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `e2e-verify-checks-out-pr-head` job of
`.github/workflows/testing-conventions-selftest.yml`.

On a `pull_request` event, `actions/checkout`'s default ref is the ephemeral merge commit — a
synthetic commit an e2e attestation (which names a real, attested code commit) can never match.
So the `e2e-verify` job's checkout must pin `github.event.pull_request.head.sha` explicitly, and
the check is block-scoped: the pin must appear *inside the `e2e-verify` job*, not merely anywhere
in the file (the same behaviour the original `awk '/^  e2e-verify:/,/^  packaging:/'` had).

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
JOB_START = re.compile(r"^  e2e-verify:")
JOB_END = re.compile(r"^  packaging:")
PIN = "github.event.pull_request.head.sha"


def extract_block(text: str, start: re.Pattern[str], end: re.Pattern[str]) -> str:
    """Return the lines from the first `start` match through the first following `end` match.

    Inclusive of both boundary lines, mirroring `awk '/start/,/end/'`. If `end` is never seen the
    block runs to end-of-text; if `start` is never seen the block is empty.
    """
    lines = text.splitlines()
    block: list[str] = []
    in_block = False
    for line in lines:
        if not in_block:
            if start.match(line):
                in_block = True
                block.append(line)
            continue
        block.append(line)
        if end.match(line):
            break
    return "\n".join(block)


def find_missing_pr_head_pin(text: str) -> Optional[str]:
    """Return an error message unless the `e2e-verify` job pins the PR head commit."""
    block = extract_block(text, JOB_START, JOB_END)
    if PIN not in block:
        return (
            "the e2e-verify job's checkout doesn't pin github.event.pull_request.head.sha — on a "
            "pull_request event it checks out the ephemeral merge ref instead, which a committed "
            "attestation's recorded commit can never match"
        )
    return None


def path_from_argv(argv: list[str], default: str) -> str:
    """The workflow file to inspect: `argv[1]` when given, else the repo-relative default."""
    return argv[1] if argv[1:] else default


def main(argv: list[str]) -> int:
    path = path_from_argv(argv, DEFAULT_PATH)
    text = Path(path).read_text()
    missing = find_missing_pr_head_pin(text)
    if missing is not None:
        print(f"::error::{missing}")
        return 1
    print("e2e-verify checks out the PR head commit")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
