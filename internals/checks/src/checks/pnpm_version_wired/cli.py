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


def pnpm_steps(text: str) -> list[list[str]]:
    """Every step chunk in `text` that uses `pnpm/action-setup` — a line opening a list item
    (`- `) through the line before the next one.

    Chunking on the step boundary, rather than scanning a fixed number of lines ahead, is what
    keeps a step that sets no version from borrowing the next step's: the two land in different
    chunks whatever the gap between them. The real steps open with `- if:` and carry their
    `uses:` a line later, so the chunk — not the `uses:` line — is the unit to search. Lines
    before the first step belong to no chunk and are dropped, which puts the workflow's own
    `version:` input declaration out of scope.
    """
    chunks: list[list[str]] = []
    for line in text.splitlines():
        if line.lstrip().startswith("- "):
            chunks.append([])
        if chunks:
            chunks[-1].append(line)
    return [chunk for chunk in chunks if any(PNPM_SETUP in line for line in chunk)]


def setup_versions(text: str) -> list[str]:
    """The `version:` value of every `pnpm/action-setup` step in `text`, in file order."""
    return [
        stripped.removeprefix("version:").strip()
        for chunk in pnpm_steps(text)
        for stripped in (line.strip() for line in chunk)
        if stripped.startswith("version:")
    ]


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
