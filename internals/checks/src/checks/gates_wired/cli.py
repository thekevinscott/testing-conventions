"""Assert the reusable workflow offers a `gates` allowlist input — repo-only (#302, #306, #321).

Backs the `tc-checks gates-wired` subcommand, the red->green wiring signal for #262: the reusable
workflow (`.github/workflows/testing-conventions.yml`) offers a `gates` input that restricts which
gates run, so a consumer driving the suite-executing gates in its own build-capable jobs can adopt
the workflow for just the static gates.

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
    if "inputs.gates" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow has no `gates` input — gate selection is all-or-nothing, so a "
            "consumer that runs the suite-executing gates in its own build-capable jobs cannot "
            "adopt the workflow for the static gates (#262)"
        )
    click.echo("reusable workflow scopes its gates to the `gates` allowlist")
