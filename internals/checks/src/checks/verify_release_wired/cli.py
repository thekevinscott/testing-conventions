"""The verify-release-wired check — repo-only (#357, #321).

Backs the `tc-checks verify-release-wired` subcommand. Publish-gating (#235) ships the runtime
before the tag moves, but the promotion itself was an unguarded deploy: a release could publish a
good binary and still move `@v0` into a workflow/binary combination the consumer surface rejects
(the packaging case), turning main/consumers red with no commit. #357 gates the move on a
version-pinned verification of the exact combination being blessed; this reads
`move-major-tag.yml` and reports the wiring's failure via the pure decision in `decide.py`.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.utils.check_failed import CheckFailed
from checks.verify_release_wired.decide import check_move_gated_on_verification

DEFAULT_MOVE_TAG = ".github/workflows/move-major-tag.yml"


@click.command()
@click.argument("move_tag", default=DEFAULT_MOVE_TAG, type=click.Path())
def cli(move_tag: str) -> None:
    path = Path(move_tag)
    error = check_move_gated_on_verification(path.read_text() if path.exists() else None)
    if error is not None:
        raise CheckFailed(error)
    click.echo("@v0 advances only after the version-pinned verification passes")
