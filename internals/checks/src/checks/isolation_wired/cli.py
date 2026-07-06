"""The isolation-wired check — repo-only (#125, #321).

Backs the `tc-checks isolation-wired` subcommand: the reusable workflow
(`.github/workflows/testing-conventions.yml`) must actually invoke `unit lint`, or the shipped
isolation rule is not enforced on the `@v0` path.

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
    if "unit lint --language" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow does not invoke `unit lint` — the shipped lint rule is "
            "not enforced on the @v0 path (#125)"
        )
    click.echo("reusable workflow invokes `unit lint`")
