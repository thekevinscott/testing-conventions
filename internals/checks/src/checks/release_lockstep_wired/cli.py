"""Assert the release config republishes both halves on a CLI change, so a consumer never runs a
new CLI against an old plugin."""
from __future__ import annotations

import tomllib
from pathlib import Path

import click

from checks.release_lockstep_wired.decide import lockstep_error
from checks.utils.check_failed import CheckFailed

RELEASE_CONFIG = "putitoutthere.toml"


@click.command()
@click.argument("config", default=RELEASE_CONFIG, type=click.Path())
def cli(config: str) -> None:
    error = lockstep_error(tomllib.loads(Path(config).read_text()).get("package", []))
    if error is not None:
        raise CheckFailed(f"the release config lets the published halves drift apart: {error}")
    click.echo("every published package rebuilds from the CLI crate, so both halves advance together")
