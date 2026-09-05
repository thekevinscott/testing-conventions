"""Assert the `e2e-verify` job's own checkout pins `github.event.pull_request.head.sha` — on a
`pull_request` the default ref is the ephemeral merge commit, which an attestation naming a real
commit can never match."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.e2e_verify_checks_out_pr_head.pins_pr_head import pins_pr_head
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not pins_pr_head(Path(workflow).read_text()):
        raise CheckFailed(
            "the e2e-verify job's checkout doesn't pin github.event.pull_request.head.sha — on a "
            "pull_request event it checks out the ephemeral merge ref instead, which a committed "
            "attestation's recorded commit can never match"
        )
    click.echo("e2e-verify checks out the PR head commit")
