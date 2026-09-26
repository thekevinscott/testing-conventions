"""The agents-md-size gate and its command — repo-only.

Holds every tracked instructions file, expanded through its `@path` imports, to a character
budget. Both reads are injected, so the gate is exercised without a repo, a subprocess, or a
filesystem. `cli` is the composition root: it declares the budget option and nothing else.
"""
from __future__ import annotations

import functools
import posixpath

import click

from checks.agents_md_size.expand import expanded
from checks.agents_md_size.fs_ops import read_text as read_one_file
from checks.agents_md_size.git_ops import tracked_paths as read_tracked_paths

# 400 lines of prose-dense instructions at roughly 75 characters a line.
MAX_CHARS = 30000

NAMES = frozenset({"AGENTS.md", "CLAUDE.md"})

_MAX_CHARS_HELP = "Characters an instructions file may hold, counted after `@path` expansion."
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
    """Exit code for `root`: 1 when a tracked instructions file is over `max_chars` expanded."""
    read = functools.partial(read_text, root)
    entries = sorted(path for path in tracked_paths(root) if posixpath.basename(path) in NAMES)
    over = [
        (entry, size)
        for entry, size in ((entry, len(expanded(entry, read))) for entry in entries)
        if size > max_chars
    ]
    if not over:
        click.echo("every instructions file fits its budget.")
        return 0
    for entry, size in over:
        click.echo(
            f"::error file={entry}::{size} characters after @-import expansion, over the "
            f"{max_chars}-character budget (docs/internals/repo.md, 'Instructions file size "
            f"budget'). {_REMEDY}"
        )
    return 1


@click.command()
@click.argument("root")
@click.option("--max-chars", type=int, default=MAX_CHARS, help=_MAX_CHARS_HELP)
def cli(root: str, max_chars: int) -> None:
    raise SystemExit(run(root, max_chars))
