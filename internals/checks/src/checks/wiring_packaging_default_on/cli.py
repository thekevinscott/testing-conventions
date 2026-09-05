"""Assert the reusable workflow gates the packaging job on detect's `packaging_dist` output."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "packaging_dist" not in Path(workflow).read_text():
        raise CheckFailed(
            "the packaging job doesn't gate on detect's `packaging_dist` — "
            "packaging isn't default-on (#186)"
        )
    click.echo("packaging is default-on (gates on packaging_dist)")
