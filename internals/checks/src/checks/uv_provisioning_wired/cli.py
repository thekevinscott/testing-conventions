"""Assert the reusable workflow provisions Python with uv alone, identically across the three
suite-executing jobs."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed
from checks.uv_provisioning_wired.decide import decide


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not decide(Path(workflow).read_text()):
        raise CheckFailed(
            "the reusable workflow does not provision Python with uv alone — an "
            "actions/setup-python + `python -m pip install` arm survives, or the python-arm "
            "provisioning steps drift across unit-coverage / coverage-changed / mutation (#399)"
        )
    click.echo("the suite-executing jobs provision Python with uv alone, identically")
