"""Assert a package whose source changed adds both a changelog fragment and a migration fragment."""
from __future__ import annotations

import click

from checks.changelog_gate.gate import run


@click.command()
@click.argument("base_sha")
@click.argument("head_sha")
def cli(base_sha: str, head_sha: str) -> None:
    raise SystemExit(run(base_sha, head_sha))
