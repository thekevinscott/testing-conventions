"""The static-gates-wired check — repo-only.

Backs the `tc-checks static-gates-wired` subcommand: asserts the reusable workflow runs the five
static gates as steps of one `static` job per language. See `docs/internals/repo.md`.
"""
from __future__ import annotations

from pathlib import Path

import click

from checks.config import REUSABLE_WORKFLOW
from checks.utils.check_failed import CheckFailed

_LEGACY_JOBS = ("colocated-test", "colocated-test-changed", "unit-lint", "integration-lint")

_REQUIRED = (
    ("  static:", "no `static:` job — the five static gates aren't consolidated into one job"),
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
    (
        "unit one-function-per-file --language",
        "the one-function-per-file gate step is missing",
    ),
    ("integration lint --language", "the integration-lint gate step is missing"),
    (
        "contains(inputs.gates, '\"colocated-test\"')",
        "the colocated-test gate-membership guard is missing",
    ),
    ("contains(inputs.gates, '\"unit-lint\"')", "the unit-lint gate-membership guard is missing"),
    (
        "contains(inputs.gates, '\"one-function-per-file\"')",
        "the one-function-per-file gate-membership guard is missing",
    ),
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
    missing required substring. Empty when the five static gates are consolidated into one `static`
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
            "the reusable workflow does not consolidate the five static gates into one `static` "
            "job (#410): " + "; ".join(problems)
        )
    click.echo("the five static gates run as steps of one `static` job per language")
