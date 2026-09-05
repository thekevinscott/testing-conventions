"""Assert the reusable workflow runs detect's derived `build_command` before the suite-executing jobs."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "needs.detect.outputs.build_command" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow has no derived `build_command` step — a native-binding "
            "consumer's compiled module is never built before the suite runs (#243/#289)"
        )
    click.echo("reusable workflow wires a build_command step")
