"""Assert the `@v0` tag move stays gated on a version-pinned verification run."""
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
