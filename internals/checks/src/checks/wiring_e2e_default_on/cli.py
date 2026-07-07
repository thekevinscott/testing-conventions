"""The wiring-e2e-default-on check — repo-only (#186; epic #302, #305, #324).

Backs the `tc-checks wiring-e2e-default-on` subcommand: it asserts the reusable workflow
(`.github/workflows/testing-conventions.yml`) gates the e2e-verify job on the detect action's
`e2e_attestation` output — so e2e verify runs by default.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
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
