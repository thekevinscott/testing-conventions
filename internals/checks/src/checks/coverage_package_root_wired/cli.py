"""Assert both coverage jobs reference detect's derived `package_root`."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.coverage_package_root_wired.decide import decide
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not decide(Path(workflow).read_text()):
        raise CheckFailed(
            "a coverage job (unit-coverage / coverage-changed) does not reference "
            "needs.detect.outputs.package_root — TS install / Python env / build_command still "
            "run at the checkout root, not the derived package root (#278)"
        )
    click.echo("unit-coverage and coverage-changed install/build at the derived package root")
