"""The rolling-release-wired check — repo-only (#235, #321).

Backs the `tc-checks rolling-release-wired` subcommand. `@v0` is a moving tag, but the reusable
workflow runs the *published* npm binary while its own file is frozen at the tag — so the tag must
advance only AFTER that binary publishes, or a consumer gets new-workflow + old-binary (the #55
`unrecognized subcommand` stranding). The two static invariants that back that guarantee are the
pure decisions in `decide.py`; this command reads the two workflow files and reports their
combined failures.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.rolling_release_wired.decide import (
    check_move_major_tag,
    check_release_no_inline_tag_move,
)
from checks.utils.check_failed import CheckFailed

DEFAULT_MOVE_TAG = ".github/workflows/move-major-tag.yml"
DEFAULT_RELEASE = ".github/workflows/release.yml"


@click.command()
@click.argument("move_tag", default=DEFAULT_MOVE_TAG, type=click.Path())
@click.argument("release", default=DEFAULT_RELEASE, type=click.Path())
def cli(move_tag: str, release: str) -> None:
    move_path = Path(move_tag)
    move_text = move_path.read_text() if move_path.exists() else None
    errors = [
        err
        for err in (
            check_move_major_tag(move_text),
            check_release_no_inline_tag_move(Path(release).read_text()),
        )
        if err is not None
    ]
    if errors:
        raise CheckFailed("; ".join(errors))
    click.echo("major tag advances only via the gated move-major-tag workflow")
