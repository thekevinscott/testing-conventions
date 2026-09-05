"""Assert the reusable workflow's unit-coverage job fans out over Rust."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "coverage_languages" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow's unit-coverage job does not fan out over rust (no "
            "`coverage_languages` matrix) — the shipped Rust coverage arm (#37) is not "
            "enforced on the @v0 path (#171)"
        )
    click.echo("reusable workflow fans unit-coverage over rust")
