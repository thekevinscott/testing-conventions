"""The packaging-build-wired check — see docs/internals/repo.md, "The self-test checks package"."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.packaging_build_wired.find_missing_wiring import find_missing_wiring
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    problem = find_missing_wiring(Path(workflow).read_text())
    if problem is not None:
        raise CheckFailed(problem)
    click.echo(
        "packaging gates on packaging_build, provisions packaging_language, builds the "
        "distribution, and scans target/package"
    )
