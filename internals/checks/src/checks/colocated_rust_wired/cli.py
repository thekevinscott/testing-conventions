"""The colocated-rust-wired check — repo-only (#274, #321).

Backs the `tc-checks colocated-rust-wired` subcommand: the reusable workflow's whole-tree
colocated-test job must fan out over rust (via `detect`'s rust-inclusive
`colocated_test_languages` set), or the shipped rust presence arm (#40, inline `#[cfg(test)]`) is
not enforced on the `@v0` path.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if "colocated_test_languages" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow's colocated-test job does not fan out over rust (no "
            "`colocated_test_languages` matrix) — the shipped rust presence arm (#40) is "
            "not enforced on the @v0 path (#274)"
        )
    click.echo("reusable workflow fans colocated-test over rust")
