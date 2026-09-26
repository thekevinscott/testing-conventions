"""The agents-md-size orchestration — repo-only.

Lists every tracked path under a root, keeps the instructions files, and reports each one whose
resolved contents exceed the budget. The git read and the size predicate are both injected, so the
orchestration is exercised without a repo and without touching a file.
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.agents_md_size.agents_md_size import agents_md_size as fits_budget
from checks.agents_md_size.instructions_files import instructions_files
from checks.utils.tracked_paths import tracked_paths as read_tracked_paths

# 16384 characters. The cap is review discipline, not model attention: below it a human still
# reads the file end to end, and above it they skim, so stale rules accumulate unnoticed. See
# docs/internals/repo.md, "Instructions file size budget".
BUDGET = 16384


def run(root: str, *, budget: int = BUDGET, tracked_paths=read_tracked_paths, fits=fits_budget) -> int:
    """Exit code for `root`: 0 when every instructions file fits, 1 with an annotation per violation."""
    over = [
        path
        for path in instructions_files(tracked_paths(root))
        if not fits(Path(root) / path, budget)
    ]
    if not over:
        click.echo("every instructions file fits the budget.")
        return 0

    for path in over:
        click.echo(
            f"::error file={path}::over the {budget}-character instructions budget, counted after "
            f"`@path` imports resolve. Move sections into reference files or skills and leave an "
            f"index of pointers behind — the remedy is relocating rules, never deleting them."
        )
    return 1
