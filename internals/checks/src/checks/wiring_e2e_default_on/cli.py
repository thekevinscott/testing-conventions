"""Assert the reusable workflow gates the e2e-verify job on detect's `e2e_attestation` output."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "e2e_attestation" not in Path(workflow).read_text():
        raise CheckFailed(
            "the e2e-verify job doesn't gate on detect's `e2e_attestation` — "
            "e2e verify isn't default-on (#186)"
        )
    click.echo("e2e verify is default-on (gates on e2e_attestation)")
