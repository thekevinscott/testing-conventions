"""Assert the packaging job references detect's derived `package_root`."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "needs.detect.outputs.package_root" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow's packaging job does not reference "
            "needs.detect.outputs.package_root — a per-package dist/ is never discovered, only a "
            "checkout-root one (#280)"
        )
    click.echo("reusable workflow's packaging job reads the derived package_root")
