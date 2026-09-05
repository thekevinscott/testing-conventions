"""The pnpm-version-wired check: every `pnpm/action-setup` step takes its `version:` from
detect's `ts_pnpm_version` output, with a fallback for a published detect that predates it."""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.pnpm_version_wired.pnpm_steps import PNPM_SETUP
from checks.pnpm_version_wired.setup_versions import setup_versions
from checks.utils.check_failed import CheckFailed

DERIVED = "needs.detect.outputs.ts_pnpm_version"
# The floor a step falls back to when detect emits nothing for the output. Matched as a
# substring, so the literal has to appear.
FALLBACK = "|| '>=11'"


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    versions = setup_versions(Path(workflow).read_text())
    if not versions:
        raise CheckFailed(
            f"no `{PNPM_SETUP}` step in the reusable workflow sets a `version:` — with neither "
            "that nor a consumer `packageManager` pin the action errors 'No pnpm version is "
            "specified' (#475)"
        )
    literals = [version for version in versions if DERIVED not in version]
    if literals:
        raise CheckFailed(
            f"{len(literals)} `{PNPM_SETUP}` step(s) pin a literal pnpm version "
            f"({', '.join(literals)}) instead of `{DERIVED}` — that conflicts with any consumer "
            "`packageManager` pin and fails the job before it installs anything (#475)"
        )
    unguarded = [version for version in versions if FALLBACK not in version]
    if unguarded:
        raise CheckFailed(
            f"{len(unguarded)} `{PNPM_SETUP}` step(s) read `{DERIVED}` with no `{FALLBACK}` "
            f"fallback ({', '.join(unguarded)}) — a published detect that predates the output "
            "hands back an empty string, and an empty `version` errors 'No pnpm version is "
            "specified' against a manifest with no pin, blocking the release (#475)"
        )
    click.echo(
        f"all {len(versions)} pnpm/action-setup steps take their version from detect, with a "
        "stale-detect fallback"
    )
