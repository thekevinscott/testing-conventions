"""Assert the reusable workflow derives hermetic (build-from-HEAD) mode — repo-only (#356, #353).

Backs the `tc-checks hermetic-wired` subcommand, the red->green wiring signal for #356. Hermetic
mode is derived, never declared: the workflow takes the hermetic path iff the calling run is this
repo's own (`github.repository` belongs to the *caller* of a reusable workflow) and no explicit
`version` was requested. There is no input — a `workflow_call` input has no visibility modifier,
so a testing-only flag would be public surface any consumer could flip. Four pieces must exist:

- the derivation guard, as an exact literal (whitespace-normalized greps are how a workflow-YAML
  mistake here would fail silently);
- a `build-cli` job, building the CLI from HEAD behind that guard;
- a local (`./.github/actions/detect`) detect step alongside the published `@v0` one;
- a `cli_command` detect output downstream jobs run in place of the hardcoded `npx` invocation.

And one must not: any `inputs.hermetic` reference is the rejected flag-shaped design creeping
back in, and fails this check outright.

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

GUARD = "github.repository == 'thekevinscott/testing-conventions' && inputs.version == ''"


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    text = Path(workflow).read_text()
    if "inputs.hermetic" in text:
        raise CheckFailed(
            "the reusable workflow references `inputs.hermetic` — hermetic mode is derived from "
            "the caller (#356), never declared by an input; remove the flag"
        )
    missing = []
    if GUARD not in text:
        missing.append(f"the derivation guard (`{GUARD}`)")
    if "build-cli:" not in text:
        missing.append("a `build-cli` job")
    if "uses: ./.github/actions/detect" not in text:
        missing.append("a local (`./.github/actions/detect`) detect step")
    if "cli_command:" not in text:
        missing.append("a `cli_command` detect output")
    if missing:
        raise CheckFailed(
            "the reusable workflow has no derived hermetic (build-from-HEAD) mode — missing "
            + ", ".join(missing)
            + " — so an in-repo caller (self-test, dogfood) can only validate the published "
            "detect/binary, not the commit under test (#356)"
        )
    click.echo("reusable workflow derives a hermetic build-from-HEAD mode")
