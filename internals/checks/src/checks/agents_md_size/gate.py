"""The agents-md-size orchestration — repo-only.

Holds every tracked instructions file, expanded through its `@path` imports, to a character
budget. Each read is injected, so the orchestration is exercised without a repo.
"""
from __future__ import annotations

import functools
import posixpath

import click

from checks.agents_md_size.closure import closure
from checks.agents_md_size.fs_ops import read_text as read_one_file
from checks.agents_md_size.git_ops import tracked_paths as read_tracked_paths

# 400 lines of prose-dense instructions at roughly 75 characters a line.
MAX_CHARS = 30000

NAMES = frozenset({"AGENTS.md", "CLAUDE.md"})

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
    """Exit code for `root`: 1 when an instructions-file closure is over `max_chars`."""
    read = functools.partial(read_text, root)
    entries = sorted(path for path in tracked_paths(root) if posixpath.basename(path) in NAMES)

    over = []
    for entry in entries:
        members = closure(entry, read)
        chars = sum(len(text) for _, text in members)
        if chars > max_chars:
            over.append((entry, members, chars))

    if not over:
        click.echo("every instructions file fits its budget.")
        return 0

    for entry, members, chars in over:
        chain = ", ".join(f"{path} ({len(text)} chars)" for path, text in members)
        click.echo(
            f"::error file={entry}::{chars} characters after @-import expansion, over the "
            f"{max_chars}-character budget (docs/internals/repo.md, 'Instructions file size "
            f"budget'). Closure: {chain}. {_REMEDY}"
        )
    return 1
