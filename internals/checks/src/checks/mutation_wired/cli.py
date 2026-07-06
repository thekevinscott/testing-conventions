"""The mutation-wired check — repo-only (#204, #321).

Backs the `tc-checks mutation-wired` subcommand: the reusable workflow
(`.github/workflows/testing-conventions.yml`) must actually invoke `unit mutation`, or the
shipped mutation rule is not enforced on the `@v0` path.

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
    if "unit mutation --language" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow does not invoke `unit mutation` — the shipped mutation "
            "rule is not enforced on the @v0 path (#204)"
        )
    click.echo("reusable workflow invokes `unit mutation`")
