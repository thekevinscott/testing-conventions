"""The static-gates-wired check — repo-only.

Backs the `tc-checks static-gates-wired` subcommand: asserts the reusable workflow runs the five
static gates as steps of one `static` job per language. See `docs/internals/repo.md`.
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.static_gates_wired.violations import violations
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    problems = violations(Path(workflow).read_text())
    if problems:
        raise CheckFailed(
            "the reusable workflow does not consolidate the five static gates into one `static` "
            "job (#410): " + "; ".join(problems)
        )
    click.echo("the five static gates run as steps of one `static` job per language")
