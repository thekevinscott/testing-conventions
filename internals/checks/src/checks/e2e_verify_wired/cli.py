"""The e2e-verify-wired check — repo-only (#173, #321).

Backs the `tc-checks e2e-verify-wired` subcommand: the reusable workflow must offer the e2e
freshness gate (#68) behind an opt-in `run_e2e` input, or consumers cannot enforce it on the
`@v0` path. Both the input (`run_e2e`) and the command (`e2e verify`) must appear.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
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
