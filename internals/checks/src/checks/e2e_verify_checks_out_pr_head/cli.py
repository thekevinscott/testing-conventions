"""The e2e-verify-checks-out-pr-head check — repo-only (#321).

Backs the `tc-checks e2e-verify-checks-out-pr-head` subcommand. On a `pull_request` event,
`actions/checkout`'s default ref is the ephemeral merge commit — a synthetic commit an e2e
attestation (which names a real, attested code commit) can never match. So the `e2e-verify` job's
checkout must pin `github.event.pull_request.head.sha` explicitly, and the check is block-scoped:
the pin must appear *inside the `e2e-verify` job*, not merely anywhere in the file.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.e2e_verify_checks_out_pr_head.block import pins_pr_head
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
