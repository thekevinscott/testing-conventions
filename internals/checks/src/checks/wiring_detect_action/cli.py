"""The wiring-detect-action check — repo-only (#185; epic #302, #305, #324).

Backs the `tc-checks wiring-detect-action` subcommand: it asserts the reusable workflow
(`.github/workflows/testing-conventions.yml`) wires detection to the tested `detect` action,
rather than running detection as inline bash off the tested engine.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

import re
from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    # `uses: <maybe-prefix>actions/detect@<ref>` — the reusable workflow pins the detect action,
    # mirroring the original grep `uses:[[:space:]]*[^[:space:]]*actions/detect@`.
    if re.search(r"uses:\s*\S*actions/detect@", Path(workflow).read_text()) is None:
        raise CheckFailed(
            "the reusable workflow doesn't use the detect action — detection still runs as "
            "inline bash, off the tested engine (#185)"
        )
    click.echo("reusable workflow detects via the detect action")
