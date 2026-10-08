"""Assert the detect action published at the major tag declares every output the reusable workflow
consumes — the one link in the chain that spans a tag, so neither side's own tests reach it."""
from __future__ import annotations

import re
from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.published_detect_contract.declared_outputs import declared_outputs
from checks.published_detect_contract.published_manifest import published_manifest
from checks.utils.check_failed import CheckFailed

PUBLISHED_REF = "v0"


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
@click.option("--ref", default=PUBLISHED_REF)
def cli(workflow: str, ref: str) -> None:
    consumed = set(re.findall(r"needs\.detect\.outputs\.([a-z0-9_]+)", Path(workflow).read_text()))
    absent = sorted(consumed - declared_outputs(published_manifest(ref)))
    if absent:
        raise CheckFailed(
            f"the detect action published at {ref} declares no "
            + ", ".join(absent)
            + f", which the reusable workflow reads as `needs.detect.outputs.<name>` — a consumer "
            f"resolving the action at {ref} hands the consuming step the empty string, and the "
            "step runs as if the value were wired"
        )
    click.echo(f"the detect action published at {ref} declares every output the workflow consumes")
