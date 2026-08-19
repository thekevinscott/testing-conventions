"""The pnpm-version-wired check — repo-only (#475).

Backs the `tc-checks pnpm-version-wired` subcommand: every `pnpm/action-setup` step in the
reusable workflow (`.github/workflows/testing-conventions.yml`) must take its `version:` from
`needs.detect.outputs.ts_pnpm_version`, never a literal.

A literal there breaks every consumer that pins `packageManager`. `action-setup` throws
`Multiple versions of pnpm specified` whenever `version` is set and the field is not
*string-equal* to it — which no real pin ever is against a range — so the job dies before
installing anything. This repo pins its floors through `engines` and carries no
`packageManager` field, so dogfooding never walks that path and cannot catch the regression at
runtime. This check stands in for it.

A standalone, colocated-tested check rather than inline `run: |` bash, for the reason
[`checks.packaging_package_root_wired`] gives: inline workflow bash is untested prose and
exposed to the `${{ }}` templating trap.
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed

PNPM_SETUP = "pnpm/action-setup"
DERIVED = "needs.detect.outputs.ts_pnpm_version"


def setup_versions(text: str) -> list[str]:
    """The `version:` value of every `pnpm/action-setup` step in `text`, in file order.

    A step's `with:` block follows its `uses:` line within a few lines; the scan stops at the
    next list item or `uses:` so a step that sets no version contributes nothing rather than
    borrowing the next step's.
    """
    versions = []
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if PNPM_SETUP not in line:
            continue
        for follower in lines[index + 1 : index + 6]:
            stripped = follower.strip()
            if stripped.startswith("version:"):
                versions.append(stripped.removeprefix("version:").strip())
                break
            if stripped.startswith("- ") or stripped.startswith("uses:"):
                break
    return versions


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
    click.echo(f"all {len(versions)} pnpm/action-setup steps take their version from detect")
