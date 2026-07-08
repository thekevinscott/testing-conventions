"""The red-path-hermetic-wired check — repo-only (#379, #356).

Backs the `tc-checks red-path-hermetic-wired` subcommand: every direct-drive failure-path job in
`.github/workflows/testing-conventions-selftest.yml` must download the `hermetic-cli` artifact and
run the HEAD binary, not the published `npx testing-conventions` — so a same-PR change to the CLI's
rule logic these jobs exercise is validated by that PR, closing the last npm-latest dependency in
the merge gate (#379). The `decide` module confirms each red-path job carries both a
`needs: [... build-cli ...]` edge and the `./.github/actions/download-hermetic-cli` step, bounded to
its own YAML region.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import SELFTEST_WORKFLOW
from checks.red_path_hermetic_wired.decide import unwired_jobs
from checks.utils.check_failed import CheckFailed


@click.command()
@click.argument("workflow", default=SELFTEST_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    unwired = unwired_jobs(Path(workflow).read_text())
    if unwired:
        raise CheckFailed(
            "these direct-drive red-path jobs don't consume the hermetic-cli artifact — "
            + ", ".join(unwired)
            + " — so they still drive the published npx binary and the merge gate depends on "
            "npm-latest for their assertions; each needs a `needs: [... build-cli ...]` edge and "
            "the ./.github/actions/download-hermetic-cli step (#379)"
        )
    click.echo("every direct-drive red-path job runs the hermetic CLI built from HEAD")
