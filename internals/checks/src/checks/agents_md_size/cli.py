"""Hold every tracked AGENTS.md / CLAUDE.md to a size budget, counted after `@path` expansion."""
from __future__ import annotations

import click

from checks.agents_md_size.gate import run
from checks.agents_md_size.limits import Limits

_BASE_HELP = "Report only on closures the diff against this ref reaches."
_WARN_LINES_HELP = "Lines a closure may hold before it warns."
_WARN_BYTES_HELP = "Bytes a closure may hold before it warns."
_MAX_LINES_HELP = "Lines a closure may hold before it fails."
_MAX_BYTES_HELP = "Bytes a closure may hold before it fails."


@click.command()
@click.argument("root")
@click.option("--base", default=None, help=_BASE_HELP)
@click.option("--warn-lines", type=int, default=Limits.warn_lines, help=_WARN_LINES_HELP)
@click.option("--warn-bytes", type=int, default=Limits.warn_bytes, help=_WARN_BYTES_HELP)
@click.option("--max-lines", type=int, default=Limits.max_lines, help=_MAX_LINES_HELP)
@click.option("--max-bytes", type=int, default=Limits.max_bytes, help=_MAX_BYTES_HELP)
def cli(root: str, base, warn_lines: int, warn_bytes: int, max_lines: int, max_bytes: int) -> None:
    raise SystemExit(run(root, Limits(warn_lines, warn_bytes, max_lines, max_bytes), base))
