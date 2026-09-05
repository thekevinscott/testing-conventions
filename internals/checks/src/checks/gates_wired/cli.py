"""Assert the reusable workflow offers a `gates` input that restricts which gates run."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "inputs.gates" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow has no `gates` input — gate selection is all-or-nothing, so a "
            "consumer that runs the suite-executing gates in its own build-capable jobs cannot "
            "adopt the workflow for the static gates (#262)"
        )
    click.echo("reusable workflow scopes its gates to the `gates` allowlist")
