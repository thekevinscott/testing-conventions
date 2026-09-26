"""The agents-md-size orchestration — repo-only.

Holds every tracked instructions file, expanded through its `@path` imports, to a line and byte
budget. Each read is injected, so the orchestration is exercised without a repo.
"""
from __future__ import annotations

import functools
from typing import Optional

import click

from checks.agents_md_size.assess import assess
from checks.agents_md_size.closure import closure
from checks.agents_md_size.diff_ops import changed_paths as read_changed_paths
from checks.agents_md_size.fs_ops import read_text as read_one_file
from checks.agents_md_size.git_ops import tracked_paths as read_tracked_paths
from checks.agents_md_size.instructions_files import instructions_files
from checks.agents_md_size.limits import Limits
from checks.agents_md_size.touching import touching

_ANNOTATION = {"fail": "error", "warn": "warning"}
_LABEL = {"fail": "budget", "warn": "soft budget"}
_EXIT = {"fail": 1}
_REMEDY = (
    "Move sections into reference files or skills; the budget is what keeps an instructions file "
    "one a reviewer reads end to end rather than skims."
)


def run(
    root: str,
    limits: Limits = Limits(),
    base: Optional[str] = None,
    *,
    tracked_paths=read_tracked_paths,
    read_text=read_one_file,
    changed_paths=read_changed_paths,
) -> int:
    """Exit code for `root`: 1 when an instructions-file closure is over its hard budget."""
    read = functools.partial(read_text, root)
    closures = [(entry, closure(entry, read)) for entry in instructions_files(tracked_paths(root))]
    if base is not None:
        closures = touching(closures, changed_paths(root, base))

    verdicts = [(entry, members, *assess(members, limits)) for entry, members in closures]
    reported = [verdict for verdict in verdicts if verdict[-1] in _ANNOTATION]
    if not reported:
        click.echo("every instructions file fits its budget.")
        return 0

    budgets = {
        "fail": (limits.max_lines, limits.max_bytes),
        "warn": (limits.warn_lines, limits.warn_bytes),
    }
    for entry, members, lines, byte_count, level in reported:
        max_lines, max_bytes = budgets[level]
        chain = ", ".join(f"{path} ({len(text.splitlines())} lines)" for path, text in members)
        click.echo(
            f"::{_ANNOTATION[level]} file={entry}::{lines} lines, {byte_count} bytes after "
            f"@-import expansion, over the {max_lines}-line / {max_bytes}-byte {_LABEL[level]} "
            f"(docs/internals/repo.md, 'Instructions file size budget'). Closure: {chain}. "
            f"{_REMEDY}"
        )
    return max(_EXIT.get(level, 0) for *_, level in reported)
