"""The coverage-rust-wired check — repo-only (#171, #321).

Backs the `tc-checks coverage-rust-wired` subcommand: the reusable workflow's unit-coverage job
must fan out over rust (via `detect`'s `coverage_languages` set), or the shipped Rust coverage
arm (#37, cargo-llvm-cov regions/lines floor) is not enforced on the `@v0` path.

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
    if "coverage_languages" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow's unit-coverage job does not fan out over rust (no "
            "`coverage_languages` matrix) — the shipped Rust coverage arm (#37) is not "
            "enforced on the @v0 path (#171)"
        )
    click.echo("reusable workflow fans unit-coverage over rust")
