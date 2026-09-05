"""Assert the reusable workflow invokes `unit lint`."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "unit lint --language" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow does not invoke `unit lint` — the shipped lint rule is "
            "not enforced on the @v0 path (#125)"
        )
    click.echo("reusable workflow invokes `unit lint`")
