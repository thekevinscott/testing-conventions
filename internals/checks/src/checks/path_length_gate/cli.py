"""Assert every tracked path under a root fits Windows's `MAX_PATH` checkout budget."""
from __future__ import annotations

import click

from checks.path_length_gate.gate import run


@click.command()
@click.argument("root")
def cli(root: str) -> None:
    raise SystemExit(run(root))
