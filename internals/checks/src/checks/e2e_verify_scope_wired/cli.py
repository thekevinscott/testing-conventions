"""The e2e-verify-scope-wired check — see docs/internals/repo.md, "The self-test checks package"."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.e2e_verify_scope_wired.find_missing_wiring import find_missing_wiring
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    problem = find_missing_wiring(Path(workflow).read_text())
    if problem is not None:
        raise CheckFailed(problem)
    click.echo(
        "e2e-verify scopes the freshness walk to inputs.source, diffs inputs.base on PRs, and "
        "appends detect's extra-scope/exclude roots"
    )
