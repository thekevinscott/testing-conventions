"""The mutation-package-root-wired check — repo-only (#279, #321).

Backs the `tc-checks mutation-package-root-wired` subcommand: the reusable workflow
(`.github/workflows/testing-conventions.yml`) `mutation` job must reference
`needs.detect.outputs.package_root` — in its install steps, its `build_command` step, and its
Rust-build cache path — or it installs and builds at the checkout root, so a per-package-lockfile
monorepo (or a non-pnpm TS package, or a uv-managed Python package) fails the job. The shared
`extract_job_block` confines the search to the `mutation` job's own YAML region — a reference in a
neighbouring job must not satisfy it.

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
from checks.utils.job_block import extract_job_block


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    block = extract_job_block(Path(workflow).read_text(), "mutation", "integration-lint")
    if "needs.detect.outputs.package_root" not in block:
        raise CheckFailed(
            "the mutation job does not reference needs.detect.outputs.package_root — it still "
            "installs/builds at the checkout root, so a per-package-lockfile monorepo (or a "
            "non-pnpm TS package, or a uv-managed Python package) fails the job (#279)"
        )
    click.echo("mutation job installs and builds from the derived package root")
