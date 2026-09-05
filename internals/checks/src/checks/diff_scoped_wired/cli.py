"""Assert the reusable workflow invokes both diff-scoped `--base` checks."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.diff_scoped_wired.decide import wires_diff_scoped
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not wires_diff_scoped(Path(workflow).read_text()):
        raise CheckFailed(
            "the reusable workflow does not run the diff-scoped `--base` checks (co-change "
            "#161 / changed-line coverage #162) — they never run in CI (#172)"
        )
    click.echo("reusable workflow runs the diff-scoped --base checks")
