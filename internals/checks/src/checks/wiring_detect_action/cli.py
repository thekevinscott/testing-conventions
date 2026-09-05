"""Assert the reusable workflow wires detection to the tested `detect` action."""
from __future__ import annotations

import re
from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    # `uses: <maybe-prefix>actions/detect@<ref>` — the reusable workflow pins the detect action.
    if re.search(r"uses:\s*\S*actions/detect@", Path(workflow).read_text()) is None:
        raise CheckFailed(
            "the reusable workflow doesn't use the detect action — detection still runs as "
            "inline bash, off the tested engine (#185)"
        )
    click.echo("reusable workflow detects via the detect action")
