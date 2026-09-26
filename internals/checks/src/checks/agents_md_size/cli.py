"""Assert every instructions file under a root fits the budget an agent loads it into."""
from __future__ import annotations

import click

from checks.agents_md_size.gate import BUDGET, run


@click.command()
@click.argument("root")
@click.option(
    "--budget",
    default=BUDGET,
    show_default=True,
    type=int,
    help="Characters an instructions file may reach, counted after `@path` imports resolve.",
)
def cli(root: str, budget: int) -> None:
    raise SystemExit(run(root, budget=budget))
