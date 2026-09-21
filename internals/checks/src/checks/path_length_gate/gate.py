"""The path-length-gate orchestration — repo-only.

Lists every tracked path under a root and reports each one that a Windows checkout can't
materialize. The git read is injected, so the orchestration is exercised without a repo.
"""
from __future__ import annotations

import click

from checks.path_length_gate.decide import BUDGET, over_budget
from checks.path_length_gate.git_ops import tracked_paths as read_tracked_paths


def run(root: str, *, tracked_paths=read_tracked_paths) -> int:
    """Exit code for `root`: 0 when every tracked path fits, 1 with an annotation per violation."""
    violations = over_budget(tracked_paths(root))
    if not violations:
        click.echo("every tracked path fits the Windows checkout budget.")
        return 0

    for path, length in violations:
        click.echo(
            f"::error file={path}::{length} characters, over the {BUDGET}-character Windows "
            f"checkout budget (docs/internals/repo.md, 'Windows checkout path budget'). Shorten "
            f"the path or move it under a shallower directory."
        )
    return 1
