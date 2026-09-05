"""Assert every direct-drive failure-path job downloads the `hermetic-cli` artifact and runs the HEAD
binary."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import SELFTEST_WORKFLOW
from checks.red_path_hermetic_wired.decide import unwired_jobs
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=SELFTEST_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    unwired = unwired_jobs(Path(workflow).read_text())
    if unwired:
        raise CheckFailed(
            "these direct-drive red-path jobs don't consume the hermetic-cli artifact — "
            + ", ".join(unwired)
            + " — so they still drive the published npx binary and the merge gate depends on "
            "npm-latest for their assertions; each needs a `needs: [... build-cli ...]` edge and "
            "the ./.github/actions/download-hermetic-cli step (#379)"
        )
    click.echo("every direct-drive red-path job runs the hermetic CLI built from HEAD")
