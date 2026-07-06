"""The github-helpers-wired check — repo-only (#237, #321, #329).

Backs the `tc-checks github-helpers-wired` subcommand: `dogfood-github-helpers.yml` holds the
repo's own `.github/` helper scripts (`detect.py`, `move_major_tag.py`) to the same package
conventions as any shipped consumer, and must invoke every Python arm — the four whole-tree arms
(unit colocated-test, unit lint, unit coverage, integration lint) plus the diff-scoped mutation
rung (`unit mutation --base`). A dropped arm would silently stop dogfooding those helpers.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import DOGFOOD_HELPERS_WORKFLOW
from checks.utils.check_failed import CheckFailed

# The five Python arms the dogfood gate must invoke. The mutation arm is diff-scoped, so its
# phrase requires the `--base` flag — a bare `unit mutation` is the whole-tree form the gate does
# not use.
REQUIRED_ARMS: list[str] = [
    "unit colocated-test",
    "unit lint",
    "unit coverage",
    "integration lint",
    "unit mutation --base",
]


def wires_github_helpers(text: str) -> bool:
    """True when the dogfood workflow text invokes all five Python arms over the .github helpers."""
    return all(arm in text for arm in REQUIRED_ARMS)


@click.command()
@click.argument("workflow", default=DOGFOOD_HELPERS_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not wires_github_helpers(Path(workflow).read_text()):
        raise CheckFailed(
            "the dogfood workflow does not run every Python arm over the .github helpers — the "
            "dogfood gate (#237) must invoke unit colocated-test, unit lint, unit coverage, "
            "integration lint, and the diff-scoped unit mutation --base"
        )
    click.echo("dogfood workflow runs all five Python arms")
