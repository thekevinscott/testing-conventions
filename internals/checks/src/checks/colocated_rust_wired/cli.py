"""Assert the reusable workflow's colocated-test job fans out over Rust."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "colocated_test_languages" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow's colocated-test job does not fan out over rust (no "
            "`colocated_test_languages` matrix) — the shipped rust presence arm (#40) is "
            "not enforced on the @v0 path (#274)"
        )
    click.echo("reusable workflow fans colocated-test over rust")
