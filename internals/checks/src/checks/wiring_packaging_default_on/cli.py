"""The wiring-packaging-default-on check — repo-only (#186; epic #302, #305, #324).

Backs the `tc-checks wiring-packaging-default-on` subcommand: it asserts the reusable workflow
(`.github/workflows/testing-conventions.yml`) gates the packaging job on the detect action's
`packaging_dist` output — so packaging runs by default.

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
    if "packaging_dist" not in Path(workflow).read_text():
        raise CheckFailed(
            "the packaging job doesn't gate on detect's `packaging_dist` — "
            "packaging isn't default-on (#186)"
        )
    click.echo("packaging is default-on (gates on packaging_dist)")
