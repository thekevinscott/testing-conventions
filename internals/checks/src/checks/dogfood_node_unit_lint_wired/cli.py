from pathlib import Path

import click

from checks.config import DOGFOOD_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=DOGFOOD_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if 'source: packages/node/src\n      gates: \'["unit-lint"]\'' not in Path(workflow).read_text():
        raise CheckFailed("dogfood does not run unit lint for packages/node/src")
    click.echo("dogfood runs unit lint for packages/node/src")
