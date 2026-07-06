"""The packaging-package-root-wired check — repo-only (#280, #321).

Backs the `tc-checks packaging-package-root-wired` subcommand: the reusable workflow
(`.github/workflows/testing-conventions.yml`) packaging job must reference
`needs.detect.outputs.package_root`, or a per-package `dist/` the detect action found is never
handed to the check step — it keeps inspecting the checkout root's `dist/`. The reference confirms
the derived package root reaches the packaging check.

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
    if "needs.detect.outputs.package_root" not in Path(workflow).read_text():
        raise CheckFailed(
            "the reusable workflow's packaging job does not reference "
            "needs.detect.outputs.package_root — a per-package dist/ is never discovered, only a "
            "checkout-root one (#280)"
        )
    click.echo("reusable workflow's packaging job reads the derived package_root")
