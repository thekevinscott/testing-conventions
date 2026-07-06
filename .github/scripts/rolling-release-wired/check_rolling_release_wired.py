#!/usr/bin/env python3
"""Assert the major tag advances only via the gated workflow — `rolling-release-wired` (#235).

Repo-only: this module exists *only in this repository*. It is NOT shipped in the published
`testing-conventions` tool; it backs the `rolling-release-wired` job of
`.github/workflows/testing-conventions-selftest.yml`, which guards the red→green signal for
#235. `@v0` is a moving tag, but the reusable workflow runs the *published* npm binary while its
own file is frozen at the tag — so the tag must advance only AFTER that binary publishes, or a
consumer gets new-workflow + old-binary (the #55 `unrecognized subcommand` stranding).

Two static invariants back that guarantee, each a pure function below:
  * `check_move_major_tag` — a dedicated `move-major-tag.yml` exists, is publish-gated
    (`workflow_run` on Release), and is concurrency-guarded; and
  * `check_release_no_inline_tag_move` — `release.yml` no longer moves the tag inline
    (`git tag -f v0`), which would re-couple the move to the publish pipeline and drop the gate.

It lives here, rather than as inline `run: |` blocks, so it carries real colocated + e2e tests.
Inline `run:` bash is untested prose and is exposed to the GitHub-Actions `${{ }}` templating
trap (a `run:` body is templated before the shell sees it, so a literal `${{ ... }}` inside a
grep pattern is silently evaluated away). A standalone Python script sidesteps both.

Stdlib only.
"""
from __future__ import annotations

import sys
from pathlib import Path
from typing import Optional

DEFAULT_MOVE_TAG = ".github/workflows/move-major-tag.yml"
DEFAULT_RELEASE = ".github/workflows/release.yml"


def check_move_major_tag(text: Optional[str]) -> Optional[str]:
    """Validate the dedicated move-major-tag workflow.

    `text` is the file's contents, or `None` when the file is absent. Returns the first failing
    reason (missing file / not publish-gated / not concurrency-guarded), else `None`.
    """
    if text is None:
        return (
            "missing .github/workflows/move-major-tag.yml — the major tag has no dedicated "
            "advance workflow (#235)"
        )
    if "workflow_run:" not in text:
        return (
            ".github/workflows/move-major-tag.yml is not gated on a workflow_run (Release "
            "completion) — @v0 could advance before the binary publishes (#235)"
        )
    if "concurrency:" not in text:
        return (
            ".github/workflows/move-major-tag.yml has no concurrency group — out-of-order tag "
            "moves are unguarded (#235)"
        )
    return None


def check_release_no_inline_tag_move(text: str) -> Optional[str]:
    """Return an error if `release.yml` still moves the major tag inline (`git tag -f v0`)."""
    if "tag -f v0" in text:
        return (
            "release.yml moves @v0 inline (`git tag -f v0`) — the move belongs in the gated "
            "move-major-tag.yml, not the publish workflow (#235)"
        )
    return None


def path_from_argv(argv: list[str], index: int, default: str) -> str:
    """The file at position `index`: `argv[index]` when given, else the repo-relative default."""
    return argv[index] if argv[index:] else default


def main(argv: list[str]) -> int:
    move_tag_path = Path(path_from_argv(argv, 1, DEFAULT_MOVE_TAG))
    release_path = Path(path_from_argv(argv, 2, DEFAULT_RELEASE))
    move_text = move_tag_path.read_text() if move_tag_path.exists() else None
    errors = [
        err
        for err in (
            check_move_major_tag(move_text),
            check_release_no_inline_tag_move(release_path.read_text()),
        )
        if err is not None
    ]
    if errors:
        for err in errors:
            print(f"::error::{err}")
        return 1
    print("major tag advances only via the gated move-major-tag workflow")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
