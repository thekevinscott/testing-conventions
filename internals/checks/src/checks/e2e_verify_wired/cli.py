"""Assert the reusable workflow offers the e2e freshness gate behind the opt-in `run_e2e` input."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.e2e_verify_wired.decide import wires_e2e_verify
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not wires_e2e_verify(Path(workflow).read_text()):
        raise CheckFailed(
            "the reusable workflow does not offer an opt-in `e2e verify` job (input `run_e2e` "
            "+ `e2e verify`) — the #68 freshness gate isn't on the @v0 path (#173)"
        )
    click.echo("reusable workflow offers opt-in e2e verify")
