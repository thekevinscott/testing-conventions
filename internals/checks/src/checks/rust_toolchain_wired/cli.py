"""Assert the reusable workflow provisions a Rust toolchain input — repo-only (#302, #306, #321).

Backs the `tc-checks rust-toolchain-wired` subcommand, the red->green wiring signal for #263: the
suite-executing jobs (unit coverage, changed-line coverage, mutation) offer a `rust_toolchain`
input that provisions cargo before `build_command` runs, so a native binding whose build compiles
a Rust core has cargo available in the python/typescript arms.

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
    if "inputs.rust_toolchain" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow has no `rust_toolchain` input — a native binding whose "
            "build_command compiles a Rust core has no cargo in the python/typescript "
            "suite-executing jobs, so the build fails before the suite runs (#263)"
        )
    click.echo("reusable workflow provisions a Rust toolchain behind `rust_toolchain`")
