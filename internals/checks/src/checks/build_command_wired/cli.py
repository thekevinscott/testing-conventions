"""The build-command-wired check — repo-only (#302 #306, #321).

Backs the `tc-checks build-command-wired` subcommand: the reusable workflow
(`.github/workflows/testing-conventions.yml`) runs a derived `build_command` step — read from
detect's `build_command` output (the package's own `[python].build_command`, #289) — before the
suite-executing jobs, so a native-binding consumer's compiled module is built before the suite
imports it.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.utils.check_failed import CheckFailed

DEFAULT_WORKFLOW = ".github/workflows/testing-conventions.yml"


def wires_build_command(text: str) -> bool:
    """True when the reusable workflow references detect's derived `build_command` output."""
    return "needs.detect.outputs.build_command" in text


@click.command()
@click.argument("workflow", default=DEFAULT_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    if not wires_build_command(Path(workflow).read_text()):
        raise CheckFailed(
            "the reusable workflow has no derived `build_command` step — a native-binding "
            "consumer's compiled module is never built before the suite runs (#243/#289)"
        )
    click.echo("reusable workflow wires a build_command step")
