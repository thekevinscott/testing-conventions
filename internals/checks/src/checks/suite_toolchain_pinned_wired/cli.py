"""The suite-toolchain-pinned-wired check — repo-only (#438).

Backs the `tc-checks suite-toolchain-pinned-wired` subcommand: the reusable workflow pins the
consumer suite toolchain (`coverage`, `pytest`) to exact `==` versions in every `uv pip install`
line, so a consumer's required check resolves the engine the workflow version names — not whatever
the index serves that day. The `decide` module holds the fact.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap.
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.suite_toolchain_pinned_wired.decide import decide, unpinned_engines
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    text = Path(workflow).read_text()
    if not decide(text):
        floating = ", ".join(unpinned_engines(text)) or "coverage/pytest not pinned at all"
        raise CheckFailed(
            "the reusable workflow does not pin the consumer suite toolchain — a `uv pip install` "
            f"line leaves a suite engine floating ({floating}), so a consumer's required check "
            "resolves whatever the index serves (#438)"
        )
    click.echo("the reusable workflow pins the consumer suite toolchain (coverage, pytest) to exact versions")
