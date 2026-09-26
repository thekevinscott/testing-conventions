"""The agents-md-size gate and its command — repo-only.

Composes the two decisions: `resolve_agents_md` turns an entry into the content an agent actually
loads, `agents_md_size` says whether that content is over budget. Both reads are injected, so the
gate is exercised without a repo, a subprocess, or a filesystem. `cli` declares the budget option
and nothing else.
"""
from __future__ import annotations

import functools
import posixpath

import click

from checks.agents_md_size.agents_md_size import agents_md_size
from checks.agents_md_size.fs_ops import read_text as read_one_file
from checks.agents_md_size.git_ops import tracked_paths as read_tracked_paths
from checks.agents_md_size.resolve_agents_md.resolve_agents_md import resolve_agents_md

# 400 lines of prose-dense instructions at roughly 75 characters a line.
MAX_CHARS = 30000

NAMES = frozenset({"AGENTS.md", "CLAUDE.md"})

_MAX_CHARS_HELP = "Characters an instructions file may hold, counted after `@path` resolution."
_REMEDY = (
    "Move sections into reference files or skills; the budget is what keeps an instructions file "
    "one a reviewer reads end to end rather than skims."
)


def run(
    root: str,
    max_chars: int = MAX_CHARS,
    *,
    tracked_paths=read_tracked_paths,
    read_text=read_one_file,
) -> int:
    """Exit code for `root`: 1 when a tracked instructions file is over `max_chars` resolved."""
    read = functools.partial(read_text, root)
    entries = sorted(path for path in tracked_paths(root) if posixpath.basename(path) in NAMES)
    resolved = ((entry, resolve_agents_md(entry, read)) for entry in entries)
    over = [
        (entry, len(content))
        for entry, content in resolved
        if agents_md_size(content, max_chars)
    ]
    if not over:
        click.echo("every instructions file fits its budget.")
        return 0
    for entry, size in over:
        click.echo(
            f"::error file={entry}::{size} characters after @-import resolution, over the "
            f"{max_chars}-character budget (docs/internals/repo.md, 'Instructions file size "
            f"budget'). {_REMEDY}"
        )
    return 1


@click.command()
@click.argument("root")
@click.option("--max-chars", type=int, default=MAX_CHARS, help=_MAX_CHARS_HELP)
def cli(root: str, max_chars: int) -> None:
    raise SystemExit(run(root, max_chars))
