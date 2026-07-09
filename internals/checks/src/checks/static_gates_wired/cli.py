"""The static-gates-wired check — repo-only (#410).

Backs the `tc-checks static-gates-wired` subcommand: the four static gates — colocated-test (with
its co-change variant), unit-lint, and integration-lint — must run as steps of one `static` job per
language rather than four separate jobs, each of which paid ~20s of setup for a sub-second scan.
The check asserts the consolidation statically: the four legacy job keys are gone, a `static:` job
fans out over the rust-inclusive `static_languages` matrix (with the stale-`@v0`
`integration_lint_languages` fallback), all four gate run lines survive with their gate-membership
guards, and the co-change step alone stays gated on the pull_request event.

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

# The four job headers the consolidation removes — none may remain as a `  <name>:` job header.
_LEGACY_JOBS = ("colocated-test", "colocated-test-changed", "unit-lint", "integration-lint")

# Each required substring, with the failure message naming what its absence means.
_REQUIRED = (
    ("  static:", "no `static:` job — the four static gates aren't consolidated into one job"),
    (
        "needs.detect.outputs.static_languages || needs.detect.outputs.integration_lint_languages",
        "the `static` job's matrix doesn't read `static_languages` with the "
        "`|| integration_lint_languages` stale-@v0 fallback",
    ),
    ("unit colocated-test --language", "the colocated-test gate step is missing"),
    (
        'unit colocated-test --language "$LANGUAGE" --base',
        "the co-change gate step (the `--base` variant) is missing",
    ),
    ("unit lint --language", "the unit-lint gate step is missing"),
    ("integration lint --language", "the integration-lint gate step is missing"),
    (
        "contains(inputs.gates, '\"colocated-test\"')",
        "the colocated-test gate-membership guard is missing",
    ),
    ("contains(inputs.gates, '\"unit-lint\"')", "the unit-lint gate-membership guard is missing"),
    (
        "contains(inputs.gates, '\"integration-lint\"')",
        "the integration-lint gate-membership guard is missing",
    ),
    (
        "!cancelled() && github.event_name == 'pull_request'",
        "the co-change step's guard doesn't carry the pull_request event under `!cancelled()`",
    ),
)


def violations(text: str) -> list[str]:
    """The wiring problems in the reusable workflow `text`: any surviving legacy job header, and any
    missing required substring. Empty when the four static gates are consolidated into one `static`
    job with every gate run line, guard, and the co-change PR gate intact.
    """
    problems = [f"the legacy `{job}` job still exists" for job in _LEGACY_JOBS if f"\n  {job}:" in text]
    problems += [message for needle, message in _REQUIRED if needle not in text]
    return problems


@click.command()
@click.argument("workflow", default=REUSABLE_WORKFLOW, type=click.Path())
def cli(workflow: str) -> None:
    problems = violations(Path(workflow).read_text())
    if problems:
        raise CheckFailed(
            "the reusable workflow does not consolidate the four static gates into one `static` "
            "job (#410): " + "; ".join(problems)
        )
    click.echo("the four static gates run as steps of one `static` job per language")
