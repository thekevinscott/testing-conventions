"""The diff-scoped-wired check — repo-only (#172, #321).

Backs the `tc-checks diff-scoped-wired` subcommand: the reusable workflow must invoke both the
commit-scoped co-change check (`unit colocated-test --base`, #161) and the changed-line coverage
check (`unit coverage --base`, #162), or neither diff-scoped rule runs in CI.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.diff_scoped_wired.decide import wires_diff_scoped
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not wires_diff_scoped(Path(workflow).read_text()):
        raise CheckFailed(
            "the reusable workflow does not run the diff-scoped `--base` checks (co-change "
            "#161 / changed-line coverage #162) — they never run in CI (#172)"
        )
    click.echo("reusable workflow runs the diff-scoped --base checks")
