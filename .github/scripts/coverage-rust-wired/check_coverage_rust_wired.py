#!/usr/bin/env python3
"""Assert the reusable workflow fans unit-coverage over Rust — `coverage-rust-wired` (#171).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `coverage-rust-wired` job of
`.github/workflows/testing-conventions-selftest.yml`, which guards the red→green signal for
#171 — the unit-coverage job's matrix must include rust (via `detect`'s `coverage_languages`
set), or the shipped Rust coverage arm (#37, cargo-llvm-cov regions/lines floor) is not enforced
on the `@v0` path.

It lives here, rather than as an inline `run: |` block, so it carries real colocated + e2e tests.
Inline `run:` bash is untested prose and is exposed to the GitHub-Actions `${{ }}` templating
trap (a `run:` body is templated before the shell sees it, so a literal `${{ ... }}` inside a
grep pattern is silently evaluated away). A standalone Python script sidesteps both.

Stdlib only.
"""
from __future__ import annotations

import sys
from pathlib import Path

DEFAULT_PATH = ".github/workflows/testing-conventions.yml"
MARKER = "coverage_languages"


def find_missing_wiring(text: str) -> str | None:
    """Return an error message if the unit-coverage matrix does not fan out over rust."""
    if MARKER not in text:
        return (
            "the reusable workflow's unit-coverage job does not fan out over rust (no "
            "`coverage_languages` matrix) — the shipped Rust coverage arm (#37) is not "
            "enforced on the @v0 path (#171)"
        )
    return None


def path_from_argv(argv: list[str], default: str) -> str:
    """The workflow file to inspect: `argv[1]` when given, else the repo-relative default."""
    return argv[1] if len(argv) > 1 else default


def main(argv: list[str]) -> int:
    path = path_from_argv(argv, DEFAULT_PATH)
    text = Path(path).read_text()
    missing = find_missing_wiring(text)
    if missing is not None:
        print(f"::error::{missing}")
        return 1
    print("reusable workflow fans unit-coverage over rust")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
