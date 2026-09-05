"""Assert hermetic (build-from-HEAD) mode is derived, caller-built, and fully wired — checked per
step and per job, since a neighbour's `CLI_COMMAND` env line or `build-cli` edge would otherwise
cover a block missing its own."""
from __future__ import annotations

import re
from pathlib import Path

import click

from checks.config import DOGFOOD_WORKFLOW, REUSABLE_WORKFLOW, SELFTEST_WORKFLOW
from checks.hermetic_wired.unwired_steps import unwired_steps
from checks.utils.check_failed import CheckFailed
from checks.utils.job_block import iter_job_blocks

GUARD = "github.repository == 'thekevinscott/testing-conventions' && inputs.version == ''"

USES_LINE = re.compile(r"^\s*uses: \./\.github/workflows/testing-conventions\.yml$", re.M)
NEEDS_BUILD_CLI = re.compile(r"^\s*needs: \[[^\]]*\bbuild-cli\b[^\]]*\]$", re.M)


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
@click.argument("callers", nargs=-1, type=click.Path())
def cli(workflow: str, callers: tuple[str, ...]) -> None:
    callers = callers or (SELFTEST_WORKFLOW, DOGFOOD_WORKFLOW)
    text = Path(workflow).read_text()
    if "inputs.hermetic" in text:
        raise CheckFailed(
            "the reusable workflow references `inputs.hermetic` — hermetic mode is derived from "
            "the caller (#356), never declared by an input; remove the flag"
        )
    if "build-cli:" in text:
        raise CheckFailed(
            "the reusable workflow declares a `build-cli` job — the hermetic build lives in the "
            "repo-only caller workflows (#356): a guarded job still renders a skipped row in "
            "every consumer's checks UI, so the consumer-facing file carries hermetic steps only"
        )
    missing = []
    if GUARD not in text:
        missing.append(f"the derivation guard (`{GUARD}`)")
    if "uses: ./.github/actions/detect" not in text:
        missing.append("a local (`./.github/actions/detect`) detect step")
    if "cli_command:" not in text:
        missing.append("a `cli_command` detect output")
    if "${CLI_COMMAND:-" not in text:
        missing.append("the `${CLI_COMMAND:-` published-CLI fallback")
    if "uses: ./.github/actions/download-hermetic-cli" not in text:
        missing.append("a `hermetic-cli` artifact download (./.github/actions/download-hermetic-cli)")
    if missing:
        raise CheckFailed(
            "the reusable workflow has no derived hermetic (build-from-HEAD) mode — missing "
            + ", ".join(missing)
            + " — so an in-repo caller (self-test, dogfood) can only validate the published "
            "detect/binary, not the commit under test (#356)"
        )
    unwired_fallback = unwired_steps(text)
    if unwired_fallback:
        raise CheckFailed(
            "the reusable workflow runs the `${CLI_COMMAND:-` published-CLI fallback in "
            + ", ".join(unwired_fallback)
            + " with no `CLI_COMMAND: ${{ needs.detect.outputs.cli_command }}` in that step's "
            "own `env:` — `CLI_COMMAND` is step-local, so those steps expand to the published "
            "binary while the file-wide fallback text stays intact: green, with Layer 1 off (#356)"
        )
    for caller in callers:
        caller_text = Path(caller).read_text()
        if "build-cli:" not in caller_text:
            raise CheckFailed(
                f"{caller} has no `build-cli` job — every repo-only caller of the reusable "
                "workflow must build and stage the hermetic-cli artifact its hermetic rule jobs "
                "download (#356)"
            )
        if "uses: ./.github/actions/build-hermetic-cli" not in caller_text:
            raise CheckFailed(
                f"{caller}'s `build-cli` job doesn't call the shared "
                "`./.github/actions/build-hermetic-cli` composite action — inlining the build "
                "steps here instead lets this caller's build drift from the other caller's (#356)"
            )
        unwired = [
            name
            for name, block in iter_job_blocks(caller_text)
            if USES_LINE.search(block) and not NEEDS_BUILD_CLI.search(block)
        ]
        if unwired:
            raise CheckFailed(
                f"{caller} calls the reusable workflow from {', '.join(unwired)} with no "
                "`needs: [... build-cli ...]` on that job — without the edge the build races "
                "the artifact download and fails flaky instead of deterministically (#356)"
            )
    click.echo("hermetic build-from-HEAD mode is derived, caller-built, and fully wired")
