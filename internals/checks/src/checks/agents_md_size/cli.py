"""Hold every tracked AGENTS.md / CLAUDE.md to a character budget, counted after `@path` expansion."""
from __future__ import annotations

import click

from checks.agents_md_size.gate import MAX_CHARS, run

_MAX_CHARS_HELP = "Characters a closure may hold before it fails."


@click.command()
@click.argument("root")
@click.option("--max-chars", type=int, default=MAX_CHARS, help=_MAX_CHARS_HELP)
def cli(root: str, max_chars: int) -> None:
    raise SystemExit(run(root, max_chars))
