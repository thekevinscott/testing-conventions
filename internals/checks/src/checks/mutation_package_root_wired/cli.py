"""Assert the `mutation` job references detect's derived `package_root` inside its own YAML region."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed
from checks.utils.extract_job_block import extract_job_block


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    block = extract_job_block(Path(workflow).read_text(), "mutation", "integration-lint")
    if "needs.detect.outputs.package_root" not in block:
        raise CheckFailed(
            "the mutation job does not reference needs.detect.outputs.package_root — it still "
            "installs/builds at the checkout root, so a per-package-lockfile monorepo (or a "
            "non-pnpm TS package, or a uv-managed Python package) fails the job (#279)"
        )
    click.echo("mutation job installs and builds from the derived package root")
