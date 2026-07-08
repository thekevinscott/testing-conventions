"""Assert the hermetic (build-from-HEAD) mode is derived, caller-built, and fully wired —
repo-only (#356, #353).

Backs the `tc-checks hermetic-wired` subcommand, the red->green wiring signal for #356. Hermetic
mode is derived, never declared: detect receives `caller_repository` (the reusable workflow's
`github.repository`, which belongs to the *caller* of a reusable workflow) and `version`, and
emits `cli_command` only when this repo gates itself with no pinned version. The build lives in
the repo-only caller workflows (self-test, dogfood), never in the consumer-facing reusable
workflow: a guarded job still renders a skipped row in every consumer's checks UI, so the
reusable workflow may carry hermetic *steps* only. In the reusable workflow, five pieces must
exist:

- the derivation guard, as an exact literal, on the detect step pair (the only YAML residue —
  which action ref runs is a scheduling decision expressions alone can make);
- a local (`./.github/actions/detect`) detect step alongside the published `@v0` one;
- a `cli_command` detect output;
- the `${CLI_COMMAND:-` fallback (transition-safe: an old `@v0` detect emits no `cli_command`,
  and the consumer path must stay byte-for-byte today's npx line);
- a `hermetic-cli` artifact download, via the shared `./.github/actions/download-hermetic-cli`
  composite action (the download + chmod trio the nine rule jobs would otherwise each repeat).

And two must not: any `inputs.hermetic` reference (the rejected flag design), and any
`build-cli:` job (the rejected consumer-visible-row design). In each caller workflow, a
`build-cli:` job must exist and every job that `uses:` the reusable workflow must carry
`needs: [... build-cli ...]` on that same job — without the edge the build races the download and
fails flaky instead of deterministically. Checked per job (`iter_job_blocks`), not as two
file-wide counts: counting `uses:` lines against `needs: [... build-cli ...]` lines separately
would pass on a false negative — an unrelated job's edge, or a duplicated edge on one job,
numerically balances a different job that's missing its edge entirely, while the race stays real.

A standalone, colocated-tested check rather than inline `run: |` bash: inline workflow bash is
untested prose and exposed to the GitHub Actions `${{ }}` templating trap (the `run:` text is
templated before the shell sees it, so a literal `${{ ... }}` in a grep pattern is silently
evaluated).
"""
from __future__ import annotations

import re
from pathlib import Path

import click

from checks.config import DOGFOOD_WORKFLOW, REUSABLE_WORKFLOW, SELFTEST_WORKFLOW
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
        missing.append("the `${CLI_COMMAND:-` npx fallback")
    if "uses: ./.github/actions/download-hermetic-cli" not in text:
        missing.append("a `hermetic-cli` artifact download (./.github/actions/download-hermetic-cli)")
    if missing:
        raise CheckFailed(
            "the reusable workflow has no derived hermetic (build-from-HEAD) mode — missing "
            + ", ".join(missing)
            + " — so an in-repo caller (self-test, dogfood) can only validate the published "
            "detect/binary, not the commit under test (#356)"
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
